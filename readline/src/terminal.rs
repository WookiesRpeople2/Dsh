use std::io::{self, Write};

pub struct RawMode {
    active: bool,
    #[cfg(unix)]
    original: libc::termios,
}

impl RawMode {
    pub fn enable() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let original = unix::read_attrs()?;
            unix::apply_raw(&original)?;
            return Ok(Self {
                active: true,
                original,
            });
        }

        #[cfg(windows)]
        {
            windows::enable_raw_mode()?;
            return Ok(Self { active: true });
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "raw mode is not supported on this platform",
            ))
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        #[cfg(unix)]
        let _ = unix::restore_attrs(&self.original);

        #[cfg(windows)]
        let _ = windows::disable_raw_mode();

        self.active = false;
    }
}

pub fn size() -> io::Result<(u16, u16)> {
    #[cfg(unix)]
    {
        return unix::terminal_size();
    }
    #[cfg(windows)]
    {
        return windows::terminal_size();
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok((80, 24))
    }
}

pub fn cursor_position() -> io::Result<(u16, u16)> {
    #[cfg(unix)]
    {
        return unix::cursor_position();
    }
    #[cfg(windows)]
    {
        return windows::cursor_position();
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok((0, 0))
    }
}

pub fn move_to_column<W: Write>(out: &mut W, col: u16) -> io::Result<()> {
    write!(out, "\r\x1b[{}G", col + 1)
}

pub fn move_to<W: Write>(out: &mut W, col: u16, row: u16) -> io::Result<()> {
    write!(out, "\x1b[{};{}H", row + 1, col + 1)
}

pub fn move_left<W: Write>(out: &mut W, count: u16) -> io::Result<()> {
    write!(out, "\x1b[{count}D")
}

pub fn move_right<W: Write>(out: &mut W, count: u16) -> io::Result<()> {
    write!(out, "\x1b[{count}C")
}

pub fn clear_line<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[K")
}

pub fn scroll_up<W: Write>(out: &mut W, lines: u16) -> io::Result<()> {
    write!(out, "\x1b[{lines}S")
}

pub fn hide_cursor<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[?25l")
}

pub fn show_cursor<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[?25h")
}

pub fn set_foreground<W: Write>(out: &mut W, color: crate::Color) -> io::Result<()> {
    write!(out, "\x1b[{}m", color.ansi_fg())
}

pub fn reset_style<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "{}", crate::style::RESET)
}

#[cfg(unix)]
mod unix {
    use std::io::{self, Read, Write};

    pub fn read_attrs() -> io::Result<libc::termios> {
        let mut termios: libc::termios = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut termios) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(termios)
    }

    pub fn apply_raw(original: &libc::termios) -> io::Result<()> {
        let mut raw = *original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
        raw.c_iflag &= !(libc::IXON | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::BRKINT);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;

        let result = unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn restore_attrs(original: &libc::termios) -> io::Result<()> {
        let result = unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, original) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn terminal_size() -> io::Result<(u16, u16)> {
        unsafe {
            let mut winsize: libc::winsize = std::mem::zeroed();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok((winsize.ws_col, winsize.ws_row))
        }
    }

    pub fn cursor_position() -> io::Result<(u16, u16)> {
        let mut out = io::stdout();
        write!(out, "\x1b[6n")?;
        out.flush()?;

        let mut buf = String::new();
        let mut byte = [0u8; 1];
        loop {
            io::stdin().read_exact(&mut byte)?;
            buf.push(byte[0] as char);
            if byte[0] == b'R' {
                break;
            }
        }

        parse_cursor_report(&buf)
    }

    fn parse_cursor_report(report: &str) -> io::Result<(u16, u16)> {
        let trimmed = report.trim_start_matches("\x1b[").trim_end_matches('R');
        let (row, col) = trimmed
            .split_once(';')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid cursor report"))?;
        let row: u16 = row
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid row"))?;
        let col: u16 = col
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid column"))?;
        Ok((col.saturating_sub(1), row.saturating_sub(1)))
    }
}

#[cfg(windows)]
mod windows {
use std::io;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle, SetConsoleMode,
        CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    pub fn enable_raw_mode() -> io::Result<()> {
        let handle = std_handle(STD_INPUT_HANDLE)?;
        let mut mode = console_mode(handle)?;
        mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
        set_console_mode(handle, mode)?;

        let out_handle = std_handle(STD_OUTPUT_HANDLE)?;
        let mut out_mode = console_mode(out_handle)?;
        out_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        set_console_mode(out_handle, out_mode)?;

        Ok(())
    }

    pub fn disable_raw_mode() -> io::Result<()> {
        let handle = std_handle(STD_INPUT_HANDLE)?;
        let mut mode = console_mode(handle)?;
        mode |= ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
        set_console_mode(handle, mode)?;
        Ok(())
    }

    pub fn terminal_size() -> io::Result<(u16, u16)> {
        let handle = std_handle(STD_OUTPUT_HANDLE)?;
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        let ok = unsafe { GetConsoleScreenBufferInfo(handle, &mut info) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }

        let width = (info.srWindow.Right - info.srWindow.Left + 1) as u16;
        let height = (info.srWindow.Bottom - info.srWindow.Top + 1) as u16;
        Ok((width, height))
    }

    pub fn cursor_position() -> io::Result<(u16, u16)> {
        let handle = std_handle(STD_OUTPUT_HANDLE)?;
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        let ok = unsafe { GetConsoleScreenBufferInfo(handle, &mut info) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok((
            info.dwCursorPosition.X as u16,
            info.dwCursorPosition.Y as u16,
        ))
    }

    fn std_handle(id: u32) -> io::Result<HANDLE> {
        let handle = unsafe { GetStdHandle(id) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }

    fn console_mode(handle: HANDLE) -> io::Result<u32> {
        let mut mode = 0u32;
        let ok = unsafe { GetConsoleMode(handle, &mut mode) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(mode)
    }

    fn set_console_mode(handle: HANDLE, mode: u32) -> io::Result<()> {
        let ok = unsafe { SetConsoleMode(handle, mode) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

use std::io::{self, Write};

pub struct RawMode {
    active: bool,
    #[cfg(unix)]
    original: String,
    #[cfg(windows)]
    original_in: u32,
    #[cfg(windows)]
    original_out: u32,
}

impl RawMode {
    pub fn enable() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let original = unix::save_attrs()?;
            unix::apply_raw()?;
            return Ok(Self {
                active: true,
                original,
            });
        }

        #[cfg(windows)]
        {
            let (original_in, original_out) = windows::enable_raw_mode()?;
            Ok(Self {
                active: true,
                original_in,
                original_out,
            })
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
        let _ = windows::restore_modes(self.original_in, self.original_out);

        self.active = false;
    }
}

pub fn size() -> io::Result<(u16, u16)> {
    if let Ok(size) = ansi::query_size() {
        return Ok(size);
    }

    #[cfg(unix)]
    {
        if let Ok(size) = unix::terminal_size() {
            return Ok(size);
        }
    }

    Ok(env_size().unwrap_or((80, 24)))
}

pub fn cursor_position() -> io::Result<(u16, u16)> {
    ansi::query_cursor_position()
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

fn env_size() -> Option<(u16, u16)> {
    let cols = std::env::var("COLUMNS").ok()?.parse().ok()?;
    let rows = std::env::var("LINES").ok()?.parse().ok()?;
    Some((cols, rows))
}

mod ansi {
    use std::io::{self, Read, Write};

    pub fn query_cursor_position() -> io::Result<(u16, u16)> {
        let report = query("\x1b[6n", b'R')?;
        parse_cursor_report(&report)
    }

    pub fn query_size() -> io::Result<(u16, u16)> {
        let report = query("\x1b[18t", b't')?;
        parse_size_report(&report)
    }

    fn query(sequence: &str, terminator: u8) -> io::Result<String> {
        let mut out = io::stdout();
        write!(out, "{sequence}")?;
        out.flush()?;

        let mut buf = String::new();
        let mut byte = [0u8; 1];
        loop {
            io::stdin().read_exact(&mut byte)?;
            buf.push(byte[0] as char);
            if byte[0] == terminator {
                break;
            }
            if buf.len() > 64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "terminal query response too long",
                ));
            }
        }
        Ok(buf)
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

    fn parse_size_report(report: &str) -> io::Result<(u16, u16)> {
        let trimmed = report.trim_start_matches("\x1b[").trim_end_matches('t');
        let mut parts = trimmed.split(';');
        let _ = parts.next();
        let rows: u16 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid size report"))?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid rows"))?;
        let cols: u16 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid size report"))?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid cols"))?;
        Ok((cols, rows))
    }
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::process::{Command, Stdio};

    pub fn save_attrs() -> io::Result<String> {
        let output = Command::new("stty")
            .arg("-g")
            .stdin(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("stty -g failed"));
        }
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_owned())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid stty settings"))
    }

    pub fn apply_raw() -> io::Result<()> {
        run_stty(&["raw", "-echo", "min", "1", "time", "0"])
    }

    pub fn restore_attrs(original: &str) -> io::Result<()> {
        run_stty(&[original])
    }

    pub fn terminal_size() -> io::Result<(u16, u16)> {
        let output = Command::new("stty")
            .arg("size")
            .stdin(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("stty size failed"));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let mut parts = text.split_whitespace();
        let rows: u16 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid stty size"))?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid rows"))?;
        let cols: u16 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid stty size"))?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid cols"))?;
        Ok((cols, rows))
    }

    fn run_stty(args: &[&str]) -> io::Result<()> {
        let status = Command::new("stty")
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other("stty failed"))
        }
    }
}

#[cfg(windows)]
mod windows {
    //! Console mode control for Windows.
    //!
    //! Enabling byte-oriented input requires the Win32 console API. Those FFI
    //! calls are inherently `unsafe` in Rust; this module is the only place they
    //! appear, and each block documents the invariants that keep it sound.
    //! Display sizing and cursor queries use ANSI escapes instead (see [`super::ansi`]).

    use std::io;
    use std::mem::MaybeUninit;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE,
        STD_OUTPUT_HANDLE, SetConsoleMode,
    };

    pub fn enable_raw_mode() -> io::Result<(u32, u32)> {
        let in_handle = std_handle(STD_INPUT_HANDLE)?;
        let out_handle = std_handle(STD_OUTPUT_HANDLE)?;

        let original_in = console_mode(in_handle)?;
        let original_out = console_mode(out_handle)?;

        let mut in_mode = original_in;
        in_mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
        set_console_mode(in_handle, in_mode)?;

        let mut out_mode = original_out;
        out_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        set_console_mode(out_handle, out_mode)?;

        Ok((original_in, original_out))
    }

    pub fn restore_modes(original_in: u32, original_out: u32) -> io::Result<()> {
        let in_handle = std_handle(STD_INPUT_HANDLE)?;
        let out_handle = std_handle(STD_OUTPUT_HANDLE)?;
        set_console_mode(in_handle, original_in)?;
        set_console_mode(out_handle, original_out)?;
        Ok(())
    }

    fn std_handle(id: u32) -> io::Result<HANDLE> {
        let handle = unsafe { GetStdHandle(id) };
        if handle.is_null() || handle == -1isize as HANDLE {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }

    fn console_mode(handle: HANDLE) -> io::Result<u32> {
        let mut mode = MaybeUninit::<u32>::uninit();
        let ok = unsafe { GetConsoleMode(handle, mode.as_mut_ptr()) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { mode.assume_init() })
    }

    fn set_console_mode(handle: HANDLE, mode: u32) -> io::Result<()> {
        let ok = unsafe { SetConsoleMode(handle, mode) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

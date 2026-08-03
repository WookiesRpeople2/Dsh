pub mod file_types;
pub mod io;

#[macro_export]
macro_rules! file_types {
    (
        $(
            $variant:ident => $ext:literal, [$($alias:literal),+ $(,)?]
        ),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum FileType {
            $($variant),+
        }

        impl FileType {
            pub const fn extension(self) -> &'static str {
                match self {
                    $(FileType::$variant => $ext),+
                }
            }

            pub fn from_name(name: &str) -> Option<FileType> {
                let name = name.to_ascii_lowercase();
                $(
                    if name == $ext $(|| name == $alias)+ {
                        return Some(FileType::$variant);
                    }
                )+
                None
            }

            pub fn name(self) -> &'static str {
                match self {
                    $(FileType::$variant => stringify!($variant)),+
                }
            }

            pub const ALL: &'static [FileType] = &[$(FileType::$variant),+];
        }
    };
}

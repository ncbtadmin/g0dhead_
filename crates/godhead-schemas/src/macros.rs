/// Defines a closed enum (Book I conventions: "Enums are closed") with the
/// exact text values persisted in the store. Parsing admits listed values
/// only — extension happens by schema version bump, never by lenient input.
macro_rules! closed_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }

            pub fn parse(s: &str) -> Result<Self, $crate::error::SchemaError> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    other => Err($crate::error::SchemaError::ValidationFailed(format!(
                        "{} is a closed enum; unknown value '{}'",
                        stringify!($name),
                        other
                    ))),
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

pub(crate) use closed_enum;

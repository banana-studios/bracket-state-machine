mod state;

pub mod prelude {
    pub use crate::state::*;

    /// Helper macro to convert a type into an enum variant with the same name.
    #[macro_export]
    macro_rules! impl_from {
        ($to:ty, $from:ident) => {
            impl From<$from> for $to {
                fn from(f: $from) -> Self {
                    Self::$from(f)
                }
            }
        };
    }
}

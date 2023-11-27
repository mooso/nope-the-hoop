pub mod message;
pub mod state;
#[cfg(feature = "async")]
pub mod stream;
pub mod sync;

pub(crate) type LenType = u16;
pub(crate) const MAX_MESSAGE_SIZE: usize = 512;

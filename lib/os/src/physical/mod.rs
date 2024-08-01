//#[cfg(feature = "esp32s3")]
#[path = "esp32s3.rs"]
pub mod hardware;

pub const HEAP_SIZE: usize = 32 * 1024;

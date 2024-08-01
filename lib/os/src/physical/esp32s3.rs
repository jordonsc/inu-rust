/// esp32s3

/// The partition table itself
pub const PARTITION_OFFSET: u32 = 0x8000;
pub const PARTITION_SIZE: usize = 0xC00;

/// NVS partition location for app config
pub const NVS_OFFSET: u32 = 0x9000;
pub const NVS_SIZE: usize = 0x4000;

/// The number of ticks per second used for real-time calculations
pub const TICKS_PER_SECOND: u64 = 16_000_000;

/// Max supported pins on the chipset
pub const MAX_PINS: u8 = 49;

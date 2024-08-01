use esp_idf_svc::sys::EspError;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum OsError {
    Generic(String),
    Pin(PinError),
    Wifi(WifiError),
    FlashStorage(FlashError),
    Parse(String),
}

#[derive(Debug)]
pub enum FlashError {
    IoFault,
    IoTimeout,
    Generic(String),
    OutOfBounds(String),
    Uninitialised,
    Corrupted,
    ChecksumMismatch,
    NotFound,
}

#[derive(Debug)]
pub enum PinError {
    InvalidPin(u8),
    PinInUse(u8),
    Generic { pin: u8, error: String },
}

impl From<Utf8Error> for OsError {
    fn from(e: Utf8Error) -> Self {
        OsError::Parse(format!("Invalid UTF-8 encoding: {:?}", e))
    }
}

impl From<serde_json::Error> for OsError {
    fn from(e: serde_json::Error) -> Self {
        OsError::Parse(format!("JSON parse error: {:?}", e))
    }
}

#[derive(Debug)]
pub enum WifiError {
    Unknown(String),
    NotInitialised,
    Disconnected,
    NoIpAllocation,
}

impl From<EspError> for OsError {
    fn from(e: EspError) -> Self {
        OsError::Generic(format!("ESP error: {:?}", e))
    }
}

impl From<EspError> for FlashError {
    fn from(e: EspError) -> Self {
        FlashError::Generic(format!("ESP error: {:?}", e))
    }
}

impl From<FlashError> for OsError {
    fn from(e: FlashError) -> Self {
        OsError::FlashStorage(e)
    }
}

impl From<PinError> for OsError {
    fn from(e: PinError) -> Self {
        OsError::Pin(e)
    }
}

impl From<WifiError> for OsError {
    fn from(e: WifiError) -> Self {
        OsError::Wifi(e)
    }
}

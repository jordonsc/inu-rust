use crate::error::{FlashError, OsError};
use crate::flash::{Flash, Readable, Writable};

const SETTINGS_PARTITION: &str = "cfg";
const SETTINGS_NAMESPACE: &str = "settings";

#[derive(Debug, Default)]
pub struct WiFi {
    pub access_point: String,
    pub password: String,
}

pub struct Settings {
    flash: Flash,
    pub device_id: String,
    pub cpu_clock: u16,
    pub wifi: WiFi,
}

impl Settings {
    /// Create a new settings object or an error
    pub fn new() -> Result<Self, FlashError> {
        let mut s = Settings {
            flash: Flash::new(SETTINGS_PARTITION, SETTINGS_NAMESPACE)?,
            device_id: String::new(),
            cpu_clock: 0,
            wifi: WiFi::default(),
        };
        s.read_settings()?;
        Ok(s)
    }
}

impl Settings {
    /// Read application settings from the NVS partition.
    pub fn read_settings(&mut self) -> Result<(), FlashError> {
        self.device_id = self
            .flash
            .read("device_id")
            .unwrap_or_else(|_| "unknown.device".into());
        self.cpu_clock = self.flash.read("clock").unwrap_or(160);
        self.wifi.access_point = self
            .flash
            .read("wifi_ap")
            .unwrap_or_else(|_| "unknown".into());
        self.wifi.password = self
            .flash
            .read("wifi_pw")
            .unwrap_or_else(|_| "unknown".into());

        Ok(())
    }

    /// Write application settings to the NVS partition.
    pub fn write_settings(&mut self) -> Result<(), OsError> {
        self.flash.write("device_id", self.device_id.clone())?;
        Ok(())
    }
}

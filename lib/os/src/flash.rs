use crate::error::FlashError;
use esp_idf_svc::nvs::{EspCustomNvsPartition, EspNvs, EspNvsPartition, NvsCustom};

pub struct Flash {
    nvs: EspNvs<NvsCustom>,
}

/// Flash storage implementation with a header to store the length of the data & an MD5 hash.
/// Intended for use on NVS partitions, but not limited to that.
impl Flash {
    pub fn new(partition: &str, namespace: &str) -> Result<Self, FlashError> {
        let partition: EspNvsPartition<NvsCustom> = EspCustomNvsPartition::take(partition).unwrap();
        let nvs: EspNvs<NvsCustom> = EspNvs::new(partition, namespace, false)?;

        Ok(Flash { nvs })
    }
}

pub trait Readable<T> {
    fn read(&self, field: &str) -> Result<T, FlashError>;
}

pub trait Writable<T> {
    fn write(&mut self, field: &str, value: T) -> Result<(), FlashError>;
}

impl Readable<String> for Flash {
    fn read(&self, field: &str) -> Result<String, FlashError> {
        let mut buffer = [0u8; 16];
        match self.nvs.get_str(field, buffer.as_mut_slice())? {
            Some(s) => Ok(s.to_string()),
            None => Err(FlashError::NotFound),
        }
    }
}

impl Readable<u16> for Flash {
    fn read(&self, field: &str) -> Result<u16, FlashError> {
        match self.nvs.get_u16(field)? {
            Some(s) => Ok(s),
            None => Err(FlashError::NotFound),
        }
    }
}

impl Writable<String> for Flash {
    fn write(&mut self, field: &str, value: String) -> Result<(), FlashError> {
        self.nvs.set_str(field, &value)?;
        Ok(())
    }
}

use esp_idf_hal::cpu;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use std::thread;
use std::time::Duration;

use crate::error::OsError;
use crate::types::{OnlineSemaphore, WifiState};

const LOG_TGT: &str = "inu.net";

pub struct Networking<'s> {
    wifi: BlockingWifi<EspWifi<'s>>,
    online: OnlineSemaphore,
}

impl<'s> Networking<'s> {
    pub fn new(wifi: BlockingWifi<EspWifi<'s>>, online: OnlineSemaphore) -> Self {
        Networking { wifi, online }
    }

    pub fn run(&mut self) -> ! {
        log::info!(target: LOG_TGT, "Networking task started on core {}", cpu::core() as i32);

        loop {
            if !self.wifi.is_connected().unwrap_or(false) {
                log::warn!(target: LOG_TGT, "WiFi down, connecting..");

                match self.connect_wifi() {
                    Ok(_) => {
                        log::info!(target: LOG_TGT, "WiFi connected");
                    }
                    Err(e) => {
                        self.set_state(WifiState::Disconnected);
                        log::error!(target: LOG_TGT, "Failed to connect to WiFi: {:?}", e);
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    fn connect_wifi(&mut self) -> Result<(), OsError> {
        self.set_state(WifiState::Connecting);
        self.wifi.start()?;
        self.wifi.connect()?;
        self.set_state(WifiState::AcquiringIp);
        log::info!(target: LOG_TGT, "Connection established to AP");
        self.wifi.wait_netif_up()?;

        match self.wifi.wifi().sta_netif().get_ip_info() {
            Ok(r) => {
                self.set_state(WifiState::Connected(r));
            }
            Err(e) => {
                log::error!(target: LOG_TGT, "Failed to get IP info: {:?}", e);
                self.set_state(WifiState::Disconnected);
                self.wifi.disconnect()?;
            }
        }
        Ok(())
    }

    fn set_state(&mut self, state: WifiState) {
        let mut online = self.online.lock().unwrap();
        *online = state;
    }
}

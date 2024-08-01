use core::str::FromStr;
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi};

use crate::error::OsError;
use crate::pin_mgr::PinManager;
use crate::settings::Settings;

const LOG_TGT: &str = "inu.kernel";

pub struct Kernel<'s> {
    pub pin_mgr: PinManager,
    settings: Settings,
    wifi: AsyncWifi<EspWifi<'s>>,
    #[allow(dead_code)]
    sysloop: EspSystemEventLoop,
}

impl<'s> Kernel<'s> {
    /// Create a new kernel instance.
    ///
    /// # Safety
    /// Singleton. Create only once.
    pub unsafe fn new() -> Self {
        let sysloop = match EspSystemEventLoop::take() {
            Ok(s) => s,
            Err(e) => {
                log::error!(target: LOG_TGT, "Failed to take system event loop: {:?}", e);
                Self::death_loop();
            }
        };

        Self {
            pin_mgr: PinManager::new(),
            settings: {
                Settings::new().unwrap_or_else(|e| {
                    log::error!(target: LOG_TGT, "Failed to read settings: {:?}", e);
                    Self::death_loop();
                })
            },
            wifi: {
                let modem = unsafe { modem::Modem::new() };
                let wifi_timer = match EspTaskTimerService::new() {
                    Ok(t) => t,
                    Err(e) => {
                        log::error!(target: LOG_TGT, "Failed to create timer service: {:?}", e);
                        Self::death_loop();
                    }
                };

                let esp_wifi = match EspWifi::new(modem, sysloop.clone(), None) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!(target: LOG_TGT, "Failed to create wifi instance: {:?}", e);
                        Self::death_loop();
                    }
                };

                match AsyncWifi::wrap(esp_wifi, sysloop.clone(), wifi_timer) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!(target: LOG_TGT, "Failed to create async wifi instance: {:?}", e);
                        Self::death_loop();
                    }
                }
            },
            sysloop,
        }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn restart() -> ! {
        log::warn!(target: LOG_TGT, "Restarting device..");
        esp_idf_svc::hal::reset::restart();
    }

    pub async fn connect_wifi(&mut self) -> Result<(), OsError> {
        let ssid = heapless::String::<32>::from_str(self.get_settings().wifi.access_point.as_str())
            .map_err(|_| OsError::Generic("Cannot parse wifi SSID".into()))?;
        let pw = heapless::String::<64>::from_str(self.get_settings().wifi.password.as_str())
            .map_err(|_| OsError::Generic("Cannot parse wifi password".into()))?;

        // TODO: make the bssid, auth method & channel configurable
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: pw,
            channel: None,
            ..Default::default()
        });

        self.wifi.set_configuration(&wifi_configuration)?;

        self.wifi.start().await?;
        log::info!(target: LOG_TGT, "Wifi started");

        self.wifi.connect().await?;
        log::info!(target: LOG_TGT, "Wifi connected");

        self.wifi.wait_netif_up().await?;
        log::info!(target: LOG_TGT, "Wifi netif up");

        Ok(())
    }

    pub fn death_loop() -> ! {
        log::error!(target: LOG_TGT, "Death loop commenced");
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

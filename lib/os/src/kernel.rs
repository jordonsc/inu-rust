use core::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use esp_idf_hal::cpu;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

use crate::error::OsError;
use crate::networking::Networking;
use crate::pin_mgr::PinManager;
use crate::settings::Settings;
use crate::types::{OnlineSemaphore, WifiState};

const LOG_TGT: &str = "inu.kernel";

pub struct Kernel {
    pub pin_mgr: PinManager,
    settings: Settings,
    online: OnlineSemaphore,
    _net_handle: JoinHandle<()>,
    _sysloop: EspSystemEventLoop,
}

impl Kernel {
    /// Create a new kernel instance.
    ///
    /// If there are any errors during the creation, the device will enter a death loop.
    ///
    /// # Safety
    /// Singleton. Create only once.
    pub unsafe fn new() -> Self {
        log::info!("Kernel running on core {}", cpu::core() as i32);

        let sysloop = match EspSystemEventLoop::take() {
            Ok(s) => s,
            Err(e) => {
                log::error!(target: LOG_TGT, "Failed to take system event loop: {:?}", e);
                Self::death_loop();
            }
        };

        let settings = Settings::new().unwrap_or_else(|e| {
            log::error!(target: LOG_TGT, "Failed to read settings: {:?}", e);
            Self::death_loop();
        });

        let modem = unsafe { modem::Modem::new() };
        let esp_wifi = match EspWifi::new(modem, sysloop.clone(), None) {
            Ok(w) => w,
            Err(e) => {
                log::error!(target: LOG_TGT, "Failed to create wifi instance: {:?}", e);
                Self::death_loop();
            }
        };

        let mut wifi = match BlockingWifi::wrap(esp_wifi, sysloop.clone()) {
            Ok(w) => w,
            Err(e) => {
                log::error!(target: LOG_TGT, "Failed to create async wifi instance: {:?}", e);
                Self::death_loop();
            }
        };

        let ssid = heapless::String::<32>::from_str(settings.wifi.access_point.as_str()).unwrap();
        let pw = heapless::String::<64>::from_str(settings.wifi.password.as_str()).unwrap();

        // TODO: make the bssid, auth method & channel configurable
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: pw,
            channel: None,
            ..Default::default()
        });

        wifi.set_configuration(&wifi_configuration)
            .unwrap_or_else(|e| {
                log::error!(target: LOG_TGT, "Failed to set wifi configuration: {:?}", e);
                Self::death_loop();
            });

        let online = Arc::new(Mutex::new(Default::default()));
        let nw_online = online.clone();

        let networking = Self::new_thread(5, Some(cpu::Core::Core1), 2048, move || {
            let mut nw = Networking::new(wifi, nw_online);
            nw.run();
        })
        .unwrap_or_else(|e| {
            log::error!(target: LOG_TGT, "Failed to start networking task: {:?}", e);
            Self::death_loop();
        });

        Self {
            pin_mgr: PinManager::new(),
            settings,
            online,
            _net_handle: networking,
            _sysloop: sysloop,
        }
    }

    /// Check if the device is online.
    pub fn is_online(&self) -> bool {
        let online = self.online.lock().unwrap();
        matches!(*online, WifiState::Connected(_))
    }

    /// Return wifi connection state.
    pub fn wifi_state(&self) -> WifiState {
        let online = self.online.lock().unwrap();
        *online
    }

    /// Borrow the device settings.
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    /// Hard restart of the device.
    pub fn restart() -> ! {
        log::warn!(target: LOG_TGT, "Restarting device..");
        esp_idf_svc::hal::reset::restart();
    }

    /// Display welcome info to the device logger.
    pub fn log_info(&self, edition: &str, build: u32) {
        log::info!(target: LOG_TGT,"--- I N U [{}] build {} ---",edition,build);
        log::info!(target: LOG_TGT, " * Device ID:      {}", self.get_settings().device_id);
        log::info!(target: LOG_TGT, " * Access Point:   {}", self.get_settings().wifi.access_point);
    }

    /// Call this when you encounter an unrecoverable error. This will halt the device.
    /// It is better to call this than to panic, a panic will typically end up in a restart-loop.
    pub fn death_loop() -> ! {
        log::error!(target: LOG_TGT, "Death loop commenced");
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    /// Creates a new thread (FreeRTOS task) with given priority, core & stack size.
    pub fn new_thread<T>(
        priority: u8,
        core: Option<cpu::Core>,
        stack_size: usize,
        f: T,
    ) -> Result<JoinHandle<()>, OsError>
    where
        T: FnOnce() + Send + 'static,
    {
        // Set the default thread configuration
        ThreadSpawnConfiguration {
            stack_size,
            priority,
            pin_to_core: core,
            ..Default::default()
        }
        .set()?;

        // Create the new thread
        let thread = std::thread::Builder::new().spawn(f)?;

        // ..and reset the default thread config
        ThreadSpawnConfiguration::default().set()?;

        Ok(thread)
    }
}

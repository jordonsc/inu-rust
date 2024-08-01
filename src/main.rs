use std::time::Duration;

use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::task::block_on;

use inu_hardware::switch::{DelayOptions, InuSwitch};
use inu_os::kernel::Kernel;

mod release;

const LOG_TGT: &str = "inu";

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut kernel = unsafe { Kernel::new() };

    log::info!(
        target: LOG_TGT,
        "--- I N U [{}] build {} ---",
        release::EDITION,
        release::BUILD
    );
    log::info!(target: LOG_TGT, " * Device ID:      {}", kernel.get_settings().device_id);
    log::info!(target: LOG_TGT, " * Access Point:   {}", kernel.get_settings().wifi.access_point);

    match block_on(kernel.connect_wifi()) {
        Ok(_) => log::info!(target: LOG_TGT, "Connected to WiFi"),
        Err(e) => log::error!(target: LOG_TGT, "Failed to connect to WiFi: {:?}", e),
    }

    let input9 = kernel.pin_mgr.get_input(9, Pull::Down).unwrap();
    let sw9 = InuSwitch::new(input9)
        .with_callback(|state| {
            log::info!("Switch 9 state: {:?}", state);
        })
        .with_delay(DelayOptions {
            min_transition_time: Some(Duration::from_millis(50)),
        });

    log::info!(target: LOG_TGT, "-- {} online --", kernel.get_settings().device_id);
    loop {
        std::thread::sleep(Duration::from_millis(10));
        sw9.poll();
    }
}

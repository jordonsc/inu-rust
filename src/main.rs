use std::time::Duration;

use embedded_svc::http::Headers;
use embedded_svc::{http::client::Client as HttpClient, utils::io};
use esp_idf_svc::hal::gpio::Pull;

use inu_hardware::switch::{DelayOptions, InuSwitch};
use inu_os::kernel::Kernel;

mod release;

const LOG_TGT: &str = "inu";

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let kernel = unsafe { Kernel::new() };
    kernel.log_info(release::EDITION, release::BUILD);

    // Sample code for GPIO input
    let input9 = kernel.pin_mgr.get_input(9, Pull::Down).unwrap();
    let sw9 = InuSwitch::new(input9)
        .with_callback(|state| {
            log::info!("Switch 9 state: {:?}", state);
        })
        .with_delay(DelayOptions::tnx_ms(10));

    // Main loop
    let mut test = false;
    log::info!(target: LOG_TGT, "-- {} online --", kernel.get_settings().device_id);
    loop {
        std::thread::sleep(Duration::from_millis(10));
        sw9.poll();

        if kernel.is_online() && !test {
            log::info!(target: LOG_TGT, "Running test connection..");
            test = true;

            use esp_idf_svc::http::client::{
                Configuration as HttpConfiguration, EspHttpConnection,
            };

            let config = &HttpConfiguration {
                crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
                ..Default::default()
            };

            let mut client = HttpClient::wrap(EspHttpConnection::new(config).unwrap());
            let mut r = client
                .get("https://example.com/")
                .unwrap()
                .submit()
                .unwrap();
            log::info!("HTTP response: {}", r.status());

            if let Some(ct) = r.content_type() {
                log::info!("Content-Type: {}", ct);
            }

            if let Some(cl) = r.content_len() {
                log::info!("Content-Length: {}", cl);
            }

            let mut buf = [0u8; 1024];
            let bytes_read = io::try_read_full(&mut r, &mut buf)
                .map_err(|e| e.0)
                .unwrap();
            log::info!("Read {} bytes", bytes_read);
            match std::str::from_utf8(&buf[0..bytes_read]) {
                Ok(body_string) => log::info!(
                    "Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    body_string
                ),
                Err(e) => log::error!("Error decoding response body: {}", e),
            };

            // Drain the remaining response bytes
            while r.read(&mut buf).unwrap() > 0 {}
        }
    }
}

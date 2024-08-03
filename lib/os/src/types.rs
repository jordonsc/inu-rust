use esp_idf_svc::ipv4::IpInfo;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum WifiState {
    #[default]
    Disconnected,
    Connecting,
    AcquiringIp,
    Connected(IpInfo),
}

pub type OnlineSemaphore = Arc<Mutex<WifiState>>;

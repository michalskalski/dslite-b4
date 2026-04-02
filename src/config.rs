use std::net::{Ipv4Addr, Ipv6Addr};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub tunnel: TunnelConfig,
    pub aftr: AftrConfig,
    pub health: HealthConfig,
}

#[derive(Deserialize, Debug)]
pub struct TunnelConfig {
    #[serde(default = "default_tunnel_name")]
    pub name: String,
    pub local_v6: Ipv6Addr,
    #[serde(default = "default_tunnel_local_v4")]
    pub local_v4: Ipv4Addr,
}

#[derive(Deserialize, Debug)]
#[serde(from = "String")]
pub enum AftrAddress {
    Ip(Ipv6Addr),
    Fqdn(String),
}

impl From<String> for AftrAddress {
    fn from(value: String) -> Self {
        if let Ok(addr) = value.parse::<Ipv6Addr>() {
            Self::Ip(addr)
        } else {
            Self::Fqdn(value)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AftrConfig {
    pub address: AftrAddress,
}

#[derive(Deserialize, Debug)]
pub struct HealthConfig {
    #[serde(default = "default_health_interval")]
    pub interval_secs: u64,
}

fn default_tunnel_name() -> String {
    "dslite0".into()
}

fn default_health_interval() -> u64 {
    30
}

fn default_tunnel_local_v4() -> Ipv4Addr {
    Ipv4Addr::new(192, 0, 0, 2)
}

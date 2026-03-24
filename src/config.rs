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
    pub local_v6: String,
    #[serde(default = "default_tunnel_local_v4")]
    pub local_v4: String,
}

#[derive(Deserialize, Debug)]
pub struct AftrConfig {
    pub address: Option<String>,
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

fn default_tunnel_local_v4() -> String {
    "192.0.0.2".into()
}

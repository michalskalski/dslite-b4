use std::net::{IpAddr, Ipv6Addr};
use thiserror::Error;
use tokio::net;

use crate::config::AftrAddress;

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("resolving AFTR address: {0}")]
    LookupFailed(#[from] std::io::Error),
    #[error("no IPv6 address found for {0}")]
    NoIpv6(String),
}

pub async fn resolve(address: &AftrAddress) -> Result<Ipv6Addr, DnsError> {
    match address {
        AftrAddress::Ip(ip) => Ok(*ip),
        AftrAddress::Fqdn(address) => {
            for addr in net::lookup_host(format!("{}:0", address)).await? {
                match addr.ip() {
                    IpAddr::V6(v6) => return Ok(v6),
                    IpAddr::V4(_) => continue,
                }
            }
            Err(DnsError::NoIpv6(address.to_string()))
        }
    }
}

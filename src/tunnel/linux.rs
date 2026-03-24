use crate::tunnel::{TunnelBackend, TunnelError};
use async_trait::async_trait;
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct LinuxBackend {
    name: String,
    local_v6: Ipv6Addr,
    remote_v6: Ipv6Addr,
    local_v4: Ipv4Addr,
    remote_v4: Ipv4Addr,
}

impl LinuxBackend {
    pub fn new(
        name: String,
        local_v6: Ipv6Addr,
        remote_v6: Ipv6Addr,
        local_v4: Ipv4Addr,
        remote_v4: Ipv4Addr,
    ) -> Self {
        Self {
            name,
            local_v6,
            remote_v6,
            local_v4,
            remote_v4,
        }
    }
}

#[async_trait]
impl TunnelBackend for LinuxBackend {
    async fn setup(&self) -> Result<(), TunnelError> {
        todo!()
    }
    async fn teardown(&self) -> Result<(), TunnelError> {
        todo!()
    }
    async fn is_up(&self) -> Result<bool, TunnelError> {
        todo!()
    }
}

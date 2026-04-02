use crate::tunnel::{TunnelBackend, TunnelError};
use futures_util::stream::TryStreamExt;
use rtnetlink::{
    Handle, LinkMessageBuilder, LinkUnspec, RouteMessageBuilder, new_connection,
    packet_route::{
        IpProtocol,
        link::{InfoData, InfoIpTunnel, InfoKind, Ip6TunnelFlags, LinkFlags},
        route::RouteScope,
    },
};
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct LinuxBackend {
    name: String,
    local_v6: Ipv6Addr,
    remote_v6: Ipv6Addr,
    local_v4: Ipv4Addr,
}

impl LinuxBackend {
    pub fn new(name: String, local_v6: Ipv6Addr, remote_v6: Ipv6Addr, local_v4: Ipv4Addr) -> Self {
        Self {
            name,
            local_v6,
            remote_v6,
            local_v4,
        }
    }

    fn open_handle() -> Result<Handle, TunnelError> {
        let (connection, handle, _) =
            new_connection().map_err(|e| TunnelError::CreationFailed(e.to_string()))?;
        tokio::spawn(connection);
        Ok(handle)
    }

    async fn get_link_index(&self, handle: &Handle) -> Result<Option<u32>, rtnetlink::Error> {
        let mut links = handle.link().get().match_name(self.name.clone()).execute();
        match links.try_next().await? {
            Some(link) => Ok(Some(link.header.index)),
            None => Ok(None),
        }
    }

    async fn create_tunnel(&self, handle: &Handle) -> Result<u32, TunnelError> {
        let message = LinkMessageBuilder::<LinkUnspec>::new_with_info_kind(InfoKind::Ip6Tnl)
            .set_info_data(InfoData::IpTunnel(vec![
                InfoIpTunnel::Local(std::net::IpAddr::V6(self.local_v6)),
                InfoIpTunnel::Remote(std::net::IpAddr::V6(self.remote_v6)),
                InfoIpTunnel::Protocol(IpProtocol::Ipip),
                InfoIpTunnel::Ipv6Flags(Ip6TunnelFlags::IgnEncapLimit), // TODO: make configurable
            ]))
            .name(self.name.clone())
            .mtu(1280)
            .up()
            .build();

        handle
            .link()
            .add(message)
            .execute()
            .await
            .map_err(|e| TunnelError::CreationFailed(e.to_string()))?;

        self.get_link_index(handle)
            .await
            .map_err(|e| TunnelError::CreationFailed(e.to_string()))?
            .ok_or_else(|| {
                TunnelError::CreationFailed(format!(
                    "interface {} not found after creation",
                    self.name
                ))
            })
            .inspect(|u| tracing::debug!(name = %self.name, index = %u, "created interface"))
    }

    async fn add_address(&self, handle: &Handle, index: u32) -> Result<(), TunnelError> {
        handle
            .address()
            .add(index, std::net::IpAddr::V4(self.local_v4), 32)
            .execute()
            .await
            .map_err(|e| TunnelError::AddressFailed(e.to_string()))
            .inspect(|_| tracing::debug!(address = %self.local_v4, "assigned local address"))
    }

    async fn add_default_route(&self, handle: &Handle, index: u32) -> Result<(), TunnelError> {
        let route = RouteMessageBuilder::<Ipv4Addr>::new()
            .output_interface(index)
            .scope(RouteScope::Link)
            .build();

        handle
            .route()
            .add(route)
            .execute()
            .await
            .map_err(|e| TunnelError::RouteFailed(e.to_string()))
            .inspect(|_| tracing::debug!("default route added"))
    }
}

impl TunnelBackend for LinuxBackend {
    async fn setup(&self) -> Result<(), TunnelError> {
        let handle = Self::open_handle()?;

        let index = self.create_tunnel(&handle).await?;
        self.add_address(&handle, index).await?;
        self.add_default_route(&handle, index).await?;

        tracing::info!(
            name = %self.name,
            local_v6 = %self.local_v6,
            remote_v6 = %self.remote_v6,
            local_v4 = %self.local_v4,
            "tunnel established"
        );

        Ok(())
    }

    async fn teardown(&self) -> Result<(), TunnelError> {
        let handle = Self::open_handle()?;

        let index = self
            .get_link_index(&handle)
            .await
            .map_err(|e| TunnelError::DestroyFailed(e.to_string()))?
            .ok_or_else(|| {
                TunnelError::DestroyFailed(format!("interface {} not found", self.name))
            })?;

        handle
            .link()
            .del(index)
            .execute()
            .await
            .map_err(|e| TunnelError::DestroyFailed(e.to_string()))
            .inspect(|_| tracing::info!(name=%self.name, "interface removed"))
    }

    async fn is_up(&self) -> Result<bool, TunnelError> {
        let handle = Self::open_handle()?;

        let mut links = handle.link().get().match_name(self.name.clone()).execute();
        match links.try_next().await {
            Ok(Some(link)) => Ok(link.header.flags.contains(LinkFlags::Up)),
            Ok(None) => Ok(false),
            Err(e) => Err(TunnelError::StatusCheckFailed(e.to_string())),
        }
    }
}

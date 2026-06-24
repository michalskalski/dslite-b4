#[cfg(target_os = "linux")]
use futures_util::StreamExt;
#[cfg(target_os = "linux")]
use rtnetlink::{
    MulticastGroup, new_multicast_connection, packet_core::NetlinkMessage,
    packet_route::RouteNetlinkMessage,
};
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
pub struct NetworkChanges {
    messages: futures_channel::mpsc::UnboundedReceiver<(
        NetlinkMessage<RouteNetlinkMessage>,
        rtnetlink::sys::SocketAddr,
    )>,
    task: tokio::task::JoinHandle<()>,
}

#[cfg(target_os = "linux")]
impl NetworkChanges {
    pub fn new() -> anyhow::Result<Self> {
        let (connection, _, messages) = new_multicast_connection(&[
            MulticastGroup::Link,
            MulticastGroup::Ipv6Ifaddr,
            MulticastGroup::Ipv6Route,
        ])?;
        let task = tokio::spawn(connection);
        Ok(Self { messages, task })
    }

    pub async fn changed(&mut self) -> anyhow::Result<()> {
        let Some((_message, _)) = self.messages.next().await else {
            return Err(anyhow::anyhow!("network-change event stream ended"));
        };

        let mut count = 1;

        while let Ok(Some((_, _))) =
            tokio::time::timeout(Duration::from_millis(100), self.messages.next()).await
        {
            count += 1;
        }

        tracing::debug!(count, "network-change hints received");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for NetworkChanges {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(target_os = "illumos")]
pub struct NetworkChanges;

#[cfg(target_os = "illumos")]
impl NetworkChanges {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub async fn changed(&mut self) -> anyhow::Result<()> {
        std::future::pending::<anyhow::Result<()>>().await
    }
}

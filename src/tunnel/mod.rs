use std::net::Ipv4Addr;
use thiserror::Error;

// RFC 6333 5.7: AFTR element reserved address
const AFTR_V4_ELEMENT: Ipv4Addr = Ipv4Addr::new(192, 0, 0, 1);
// RFC 6333 5.7: B4 elements live in 192.0.0.0/29 (B4 hosts at .2..=.6,
// AFTR at .1, .0 subnet, .7 broadcast).
const B4_V4_PREFIX_LEN: u8 = 29;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "illumos")]
pub mod illumos;

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("creating tunnel: {0}")]
    CreationFailed(String),
    #[error("destroying tunnel: {0}")]
    DestroyFailed(String),
    #[error("assigning address: {0}")]
    AddressFailed(String),
    #[error("setting route: {0}")]
    RouteFailed(String),
    #[error("checking tunnel status: {0}")]
    StatusCheckFailed(String),
}

pub trait TunnelBackend: Send + Sync {
    fn setup(&self) -> impl Future<Output = Result<(), TunnelError>> + Send;
    fn teardown(&self) -> impl Future<Output = Result<(), TunnelError>> + Send;
    fn is_up(&self) -> impl Future<Output = Result<bool, TunnelError>> + Send;
}

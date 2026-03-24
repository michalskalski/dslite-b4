use async_trait::async_trait;
use thiserror::Error;

#[cfg(target_os = "linux")]
pub mod linux;

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

#[async_trait]
pub trait TunnelBackend {
    async fn setup(&self) -> Result<(), TunnelError>;
    async fn teardown(&self) -> Result<(), TunnelError>;
    async fn is_up(&self) -> Result<bool, TunnelError>;
}

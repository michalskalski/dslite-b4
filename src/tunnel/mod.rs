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

pub trait TunnelBackend: Send + Sync {
    fn setup(&self) -> impl Future<Output = Result<(), TunnelError>> + Send;
    fn teardown(&self) -> impl Future<Output = Result<(), TunnelError>> + Send;
    fn is_up(&self) -> impl Future<Output = Result<bool, TunnelError>> + Send;
}

pub mod service_registry;
pub mod message;
pub mod transport;

pub use service_registry::ServiceRegistry;
pub use message::*;
pub use transport::Transport;

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::RwLock;

pub type Result<T> = std::result::Result<T, BusError>;

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("service not found: {0}")]
    ServiceNotFound(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("timeout: {0}")]
    Timeout(String),
}

pub type ServiceId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    pub id: String,
    pub source: ServiceId,
    pub target: Option<ServiceId>,
    pub kind: MessageKind,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

pub struct RetroBus {
    pub registry: Arc<RwLock<ServiceRegistry>>,
    pub transport: Box<dyn Transport>,
}

impl RetroBus {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(ServiceRegistry::new())),
            transport,
        }
    }
}

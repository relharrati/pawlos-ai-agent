pub mod client;
pub mod types;
pub mod registry;
pub mod catalog;

pub use client::ProviderClient;
pub use types::{ChatRequest, ChatResponse, StreamChunk, LlmMessage};
pub use registry::ProviderRegistry;
pub use catalog::{ModelCatalog, LocalModelDetector, ModelFallback, DeviceCapabilities, DeviceTier};

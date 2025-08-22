//! Vehicle Nanomsg Core Library
//! 
//! 高性能车辆消息处理核心库，用于替代Python实现的nanomsg消息处理逻辑

pub mod types;
pub mod message_processor;
pub mod nanomsg_client;
pub mod performance;
pub mod error;

#[cfg(test)]
mod tests;

// 重新导出主要类型
pub use types::*;
pub use message_processor::MessageProcessor;
pub use nanomsg_client::{NanomsgClient, NanomsgConfig, ConnectionState};
pub use performance::{PerformanceMonitor, HealthStatus};
pub use error::{VehicleError, Result};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取库信息
pub fn get_library_info() -> String {
    format!(
        "Vehicle NN Core v{} - High-performance vehicle message processing library",
        VERSION
    )
}
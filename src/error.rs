use thiserror::Error;

/// 车辆消息处理相关错误类型
#[derive(Error, Debug)]
pub enum VehicleError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Message queue full")]
    QueueFull,
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Nanomsg error: {0}")]
    NanomsgError(String),
    
    #[error("Processing timeout")]
    Timeout,
    
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// 统一的Result类型
pub type Result<T> = std::result::Result<T, VehicleError>;

impl VehicleError {
    /// 检查是否为可恢复的错误
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            VehicleError::QueueFull | 
            VehicleError::Timeout |
            VehicleError::NanomsgError(_)
        )
    }
}
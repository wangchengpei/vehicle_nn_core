use crate::error::{Result, VehicleError};
use crate::message_processor::MessageProcessor;

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use parking_lot::RwLock;
use tracing::{info, warn, error};

/// Nanomsg客户端配置
#[derive(Debug, Clone)]
pub struct NanomsgConfig {
    /// 监听URL
    pub listen_url: String,
    /// 接收超时时间
    pub receive_timeout: Duration,
    /// 重连间隔
    pub reconnect_interval: Duration,
    /// 最大重连次数
    pub max_reconnect_attempts: u32,
    /// 接收缓冲区大小
    pub buffer_size: usize,
    /// 批量接收大小
    pub batch_size: usize,
    /// 批量接收超时
    pub batch_timeout: Duration,
}

impl Default for NanomsgConfig {
    fn default() -> Self {
        Self {
            listen_url: "ipc:///tmp/vehicle_nn.ipc".to_string(),
            receive_timeout: Duration::from_millis(100),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 10,
            buffer_size: 8192,
            batch_size: 100,
            batch_timeout: Duration::from_millis(10),
        }
    }
}

/// Nanomsg连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// 模拟的Nanomsg Socket（实际实现需要真正的nanomsg绑定）
pub struct MockNanomsgSocket {
    url: String,
    is_connected: bool,
    message_count: u64,
}

impl MockNanomsgSocket {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            is_connected: false,
            message_count: 0,
        }
    }
    
    pub fn bind(&mut self, url: &str) -> Result<()> {
        self.url = url.to_string();
        self.is_connected = true;
        info!("Mock nanomsg socket bound to: {}", url);
        Ok(())
    }
    
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if !self.is_connected {
            return Err(VehicleError::NanomsgError("Socket not connected".to_string()));
        }
        
        // 模拟接收消息
        self.message_count += 1;
        
        // 每10个消息中有1个是空的（模拟无消息情况）
        if self.message_count % 10 == 0 {
            return Err(VehicleError::NanomsgError("No message available".to_string()));
        }
        
        // 生成模拟消息
        let mock_message = format!(
            r#"{{
                "service": "{}",
                "params": {{
                    "vin": "MOCK_VIN_{}",
                    "timestamp": {},
                    "data": {{"x": {}, "y": {}, "speed": {}}}
                }}
            }}"#,
            if self.message_count % 5 == 0 { "tracking" } else { "traj" },
            self.message_count % 3,
            chrono::Utc::now().timestamp(),
            self.message_count as f64 * 0.1,
            self.message_count as f64 * 0.2,
            30.0 + (self.message_count % 20) as f64
        );
        
        let bytes = mock_message.as_bytes();
        let copy_len = std::cmp::min(bytes.len(), buffer.len());
        buffer[..copy_len].copy_from_slice(&bytes[..copy_len]);
        
        Ok(copy_len)
    }
    
    pub fn close(&mut self) {
        self.is_connected = false;
        info!("Mock nanomsg socket closed");
    }
}

/// 高性能Nanomsg客户端
pub struct NanomsgClient {
    config: NanomsgConfig,
    socket: Arc<RwLock<Option<MockNanomsgSocket>>>,
    message_processor: Arc<MessageProcessor>,
    connection_state: Arc<RwLock<ConnectionState>>,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<NanomsgStats>>,
}

/// Nanomsg客户端统计信息
#[derive(Debug, Clone, Default)]
pub struct NanomsgStats {
    pub bytes_received: u64,
    pub messages_received: u64,
    pub connection_attempts: u32,
    pub reconnections: u32,
    pub last_message_time: Option<Instant>,
    pub avg_batch_size: f64,
}

impl NanomsgClient {
    /// 创建新的Nanomsg客户端
    pub fn new(config: NanomsgConfig, message_processor: Arc<MessageProcessor>) -> Self {
        Self {
            config,
            socket: Arc::new(RwLock::new(None)),
            message_processor,
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(NanomsgStats::default())),
        }
    }
    
    /// 启动客户端
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.write();
            if *running {
                return Err(VehicleError::ConfigError("Client already running".to_string()));
            }
            *running = true;
        }
        
        info!("Starting Nanomsg client on: {}", self.config.listen_url);
        
        // 启动连接管理任务
        let connection_task = self.spawn_connection_manager();
        
        // 启动消息接收任务
        let receiver_task = self.spawn_message_receiver();
        
        // 启动统计报告任务
        let stats_task = self.spawn_stats_reporter();
        
        // 等待任务完成
        tokio::select! {
            result = connection_task => {
                error!("Connection manager task ended: {:?}", result);
            }
            result = receiver_task => {
                error!("Message receiver task ended: {:?}", result);
            }
            result = stats_task => {
                error!("Stats reporter task ended: {:?}", result);
            }
        }
        
        Ok(())
    }
    
    /// 停止客户端
    pub fn stop(&self) {
        info!("Stopping Nanomsg client");
        
        {
            let mut running = self.is_running.write();
            *running = false;
        }
        
        // 关闭socket
        if let Some(mut socket) = self.socket.write().take() {
            socket.close();
        }
        
        {
            let mut state = self.connection_state.write();
            *state = ConnectionState::Disconnected;
        }
    }
    
    /// 生成连接管理任务
    fn spawn_connection_manager(&self) -> tokio::task::JoinHandle<Result<()>> {
        let config = self.config.clone();
        let socket = self.socket.clone();
        let connection_state = self.connection_state.clone();
        let is_running = self.is_running.clone();
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            info!("Started connection manager");
            
            while *is_running.read() {
                let current_state = *connection_state.read();
                
                match current_state {
                    ConnectionState::Disconnected => {
                        // 尝试连接
                        {
                            let mut state = connection_state.write();
                            *state = ConnectionState::Connecting;
                        }
                        
                        match Self::establish_connection(&config, &socket, &stats).await {
                            Ok(_) => {
                                let mut state = connection_state.write();
                                *state = ConnectionState::Connected;
                                info!("Successfully connected to: {}", config.listen_url);
                            }
                            Err(e) => {
                                let mut state = connection_state.write();
                                *state = ConnectionState::Error;
                                error!("Failed to connect: {}", e);
                            }
                        }
                    }
                    ConnectionState::Error => {
                        // 等待重连间隔
                        sleep(config.reconnect_interval).await;
                        
                        let mut state = connection_state.write();
                        *state = ConnectionState::Disconnected;
                    }
                    _ => {
                        // 连接正常，检查连接状态
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            
            info!("Connection manager stopped");
            Ok(())
        })
    }
    
    /// 建立连接
    async fn establish_connection(
        config: &NanomsgConfig,
        socket: &Arc<RwLock<Option<MockNanomsgSocket>>>,
        stats: &Arc<RwLock<NanomsgStats>>,
    ) -> Result<()> {
        let mut attempts = 0;
        
        while attempts < config.max_reconnect_attempts {
            attempts += 1;
            
            {
                let mut stats_guard = stats.write();
                stats_guard.connection_attempts += 1;
            }
            
            match Self::try_connect(config).await {
                Ok(new_socket) => {
                    let mut socket_guard = socket.write();
                    *socket_guard = Some(new_socket);
                    
                    if attempts > 1 {
                        let mut stats_guard = stats.write();
                        stats_guard.reconnections += 1;
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!("Connection attempt {} failed: {}", attempts, e);
                    if attempts < config.max_reconnect_attempts {
                        sleep(config.reconnect_interval).await;
                    }
                }
            }
        }
        
        Err(VehicleError::NanomsgError(
            format!("Failed to connect after {} attempts", config.max_reconnect_attempts)
        ))
    }
    
    /// 尝试连接
    async fn try_connect(config: &NanomsgConfig) -> Result<MockNanomsgSocket> {
        let mut socket = MockNanomsgSocket::new();
        socket.bind(&config.listen_url)?;
        
        // 模拟连接延迟
        sleep(Duration::from_millis(10)).await;
        
        Ok(socket)
    }
    
    /// 生成消息接收任务
    fn spawn_message_receiver(&self) -> tokio::task::JoinHandle<Result<()>> {
        let config = self.config.clone();
        let socket = self.socket.clone();
        let message_processor = self.message_processor.clone();
        let connection_state = self.connection_state.clone();
        let is_running = self.is_running.clone();
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            info!("Started message receiver");
            let mut buffer = vec![0u8; config.buffer_size];
            
            while *is_running.read() {
                let current_state = *connection_state.read();
                
                if current_state != ConnectionState::Connected {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
                
                // 批量接收消息
                match Self::receive_message_batch(
                    &config,
                    &socket,
                    &message_processor,
                    &stats,
                    &mut buffer,
                ).await {
                    Ok(count) => {
                        if count == 0 {
                            // 没有消息，短暂休眠
                            sleep(Duration::from_micros(100)).await;
                        }
                    }
                    Err(e) => {
                        error!("Message receiving error: {}", e);
                        
                        // 连接可能断开，更新状态
                        {
                            let mut state = connection_state.write();
                            *state = ConnectionState::Error;
                        }
                        
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
            
            info!("Message receiver stopped");
            Ok(())
        })
    }
    
    /// 批量接收消息
    async fn receive_message_batch(
        config: &NanomsgConfig,
        socket: &Arc<RwLock<Option<MockNanomsgSocket>>>,
        message_processor: &Arc<MessageProcessor>,
        stats: &Arc<RwLock<NanomsgStats>>,
        buffer: &mut [u8],
    ) -> Result<usize> {
        let batch_start = Instant::now();
        let mut message_count = 0;
        
        // 在指定时间内尽可能多地接收消息
        while message_count < config.batch_size && 
              batch_start.elapsed() < config.batch_timeout {
            
            let receive_result = {
                let mut socket_guard = socket.write();
                if let Some(ref mut sock) = socket_guard.as_mut() {
                    sock.recv(buffer)
                } else {
                    return Err(VehicleError::NanomsgError("Socket not available".to_string()));
                }
            };
            
            match receive_result {
                Ok(bytes_received) => {
                    if bytes_received > 0 {
                        // 提交消息给处理器
                        if let Err(e) = message_processor.submit_message(&buffer[..bytes_received]).await {
                            warn!("Failed to submit message: {}", e);
                        } else {
                            message_count += 1;
                            
                            // 更新统计
                            {
                                let mut stats_guard = stats.write();
                                stats_guard.bytes_received += bytes_received as u64;
                                stats_guard.messages_received += 1;
                                stats_guard.last_message_time = Some(Instant::now());
                            }
                        }
                    }
                }
                Err(VehicleError::NanomsgError(_)) => {
                    // 没有消息可接收，退出批量接收
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        // 更新平均批量大小
        if message_count > 0 {
            let mut stats_guard = stats.write();
            if stats_guard.avg_batch_size == 0.0 {
                stats_guard.avg_batch_size = message_count as f64;
            } else {
                // 移动平均
                stats_guard.avg_batch_size = 
                    (stats_guard.avg_batch_size * 0.9) + (message_count as f64 * 0.1);
            }
        }
        
        Ok(message_count)
    }
    
    /// 生成统计报告任务
    fn spawn_stats_reporter(&self) -> tokio::task::JoinHandle<Result<()>> {
        let stats = self.stats.clone();
        let is_running = self.is_running.clone();
        let connection_state = self.connection_state.clone();
        
        tokio::spawn(async move {
            info!("Started stats reporter");
            
            while *is_running.read() {
                sleep(Duration::from_secs(30)).await;
                
                let stats_snapshot = stats.read().clone();
                let current_state = *connection_state.read();
                
                info!(
                    "Nanomsg Stats - State: {:?}, Messages: {}, Bytes: {}, \
                     Connections: {}, Reconnections: {}, Avg Batch: {:.1}",
                    current_state,
                    stats_snapshot.messages_received,
                    stats_snapshot.bytes_received,
                    stats_snapshot.connection_attempts,
                    stats_snapshot.reconnections,
                    stats_snapshot.avg_batch_size
                );
                
                // 检查连接健康状态
                if let Some(last_msg_time) = stats_snapshot.last_message_time {
                    let silence_duration = last_msg_time.elapsed();
                    if silence_duration > Duration::from_secs(60) {
                        warn!(
                            "No messages received for {:.1} seconds",
                            silence_duration.as_secs_f64()
                        );
                    }
                }
            }
            
            info!("Stats reporter stopped");
            Ok(())
        })
    }
    
    /// 获取连接状态
    pub fn get_connection_state(&self) -> ConnectionState {
        *self.connection_state.read()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> NanomsgStats {
        self.stats.read().clone()
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        *self.is_running.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_processor::MessageProcessor;
    
    #[tokio::test]
    async fn test_nanomsg_client_creation() {
        let config = NanomsgConfig::default();
        let processor = Arc::new(MessageProcessor::new());
        let client = NanomsgClient::new(config, processor);
        
        assert!(!client.is_running());
        assert_eq!(client.get_connection_state(), ConnectionState::Disconnected);
    }
    
    #[tokio::test]
    async fn test_mock_socket() {
        let mut socket = MockNanomsgSocket::new();
        
        // 测试绑定
        let result = socket.bind("ipc:///tmp/test.ipc");
        assert!(result.is_ok());
        
        // 测试接收
        let mut buffer = vec![0u8; 1024];
        let result = socket.recv(&mut buffer);
        assert!(result.is_ok());
        
        let bytes_received = result.unwrap();
        assert!(bytes_received > 0);
        
        // 验证接收到的是有效JSON
        let message_str = std::str::from_utf8(&buffer[..bytes_received]).unwrap();
        let _: serde_json::Value = serde_json::from_str(message_str).unwrap();
    }
    
    #[test]
    fn test_nanomsg_config() {
        let config = NanomsgConfig::default();
        
        assert!(!config.listen_url.is_empty());
        assert!(config.receive_timeout > Duration::ZERO);
        assert!(config.buffer_size > 0);
        assert!(config.batch_size > 0);
    }
}
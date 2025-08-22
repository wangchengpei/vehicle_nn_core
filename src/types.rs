use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 车辆消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleMessage {
    /// 服务类型 (tracking, route, traj, etc.)
    pub service: String,
    /// 车辆VIN码
    pub vin: String,
    /// 消息时间戳
    pub timestamp: f64,
    /// 消息参数
    pub params: HashMap<String, serde_json::Value>,
    /// 消息通道
    pub channel: String,
    /// 运行场景
    pub run_scene: Option<String>,
}

impl VehicleMessage {
    /// 创建新的车辆消息
    pub fn new(service: String, vin: String, timestamp: f64) -> Self {
        Self {
            service,
            vin,
            timestamp,
            params: HashMap::new(),
            channel: String::new(),
            run_scene: None,
        }
    }
    
    /// 获取消息的唯一标识符（用于去重）
    pub fn get_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.service.hash(&mut hasher);
        self.vin.hash(&mut hasher);
        (self.timestamp as u64).hash(&mut hasher);
        
        // 对关键参数进行hash
        if let Some(data) = self.params.get("data") {
            format!("{:?}", data).hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    /// 检查消息是否有效
    pub fn is_valid(&self) -> bool {
        !self.service.is_empty() && 
        !self.vin.is_empty() && 
        self.timestamp > 0.0 &&
        self.params.contains_key("data")
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessagePriority {
    /// 关键消息：tracking, route, error_info
    Critical,
    /// 普通消息：vcc, uos_config
    Normal,
    /// 背景消息：traj, moving_obj, device
    Background,
}

impl MessagePriority {
    /// 根据服务类型确定优先级
    pub fn from_service(service: &str) -> Self {
        match service {
            "tracking" | "route" | "error_info" => MessagePriority::Critical,
            "traj" | "moving_obj" | "device" | "loc_stat" => MessagePriority::Background,
            _ => MessagePriority::Normal,
        }
    }
    
    /// 获取队列容量
    pub fn queue_capacity(&self) -> usize {
        match self {
            MessagePriority::Critical => 200,   // 关键消息队列较大
            MessagePriority::Normal => 500,     // 普通消息队列最大
            MessagePriority::Background => 100, // 背景消息队列较小
        }
    }
    
    /// 获取处理间隔
    pub fn processing_interval(&self) -> Duration {
        match self {
            MessagePriority::Critical => Duration::from_micros(100),  // 100μs
            MessagePriority::Normal => Duration::from_millis(1),      // 1ms
            MessagePriority::Background => Duration::from_millis(10), // 10ms
        }
    }
}

/// 处理统计信息
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// 接收到的消息总数
    pub messages_received: u64,
    /// 已处理的消息数
    pub messages_processed: u64,
    /// 丢弃的消息数
    pub messages_dropped: u64,
    /// 平均处理时间（微秒）
    pub avg_processing_time_us: u64,
    /// 当前队列大小
    pub queue_size: usize,
    /// 最后更新时间
    pub last_update: Option<Instant>,
}

impl ProcessingStats {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self {
            last_update: Some(Instant::now()),
            ..Default::default()
        }
    }
    
    /// 增加接收计数
    pub fn increment_received(&mut self) {
        self.messages_received += 1;
        self.last_update = Some(Instant::now());
    }
    
    /// 增加处理计数
    pub fn increment_processed(&mut self) {
        self.messages_processed += 1;
        self.last_update = Some(Instant::now());
    }
    
    /// 增加丢弃计数
    pub fn increment_dropped(&mut self) {
        self.messages_dropped += 1;
        self.last_update = Some(Instant::now());
    }
    
    /// 更新处理时间
    pub fn update_processing_time(&mut self, duration: Duration) {
        let new_time_us = duration.as_micros() as u64;
        
        // 使用移动平均计算
        if self.avg_processing_time_us == 0 {
            self.avg_processing_time_us = new_time_us;
        } else {
            // 权重为0.1的移动平均
            self.avg_processing_time_us = 
                (self.avg_processing_time_us * 9 + new_time_us) / 10;
        }
        
        self.last_update = Some(Instant::now());
    }
    
    /// 更新队列大小
    pub fn update_queue_size(&mut self, size: usize) {
        self.queue_size = size;
        self.last_update = Some(Instant::now());
    }
    
    /// 获取处理速率（消息/秒）
    pub fn get_processing_rate(&self) -> f64 {
        if let Some(last_update) = self.last_update {
            let elapsed = last_update.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                return self.messages_processed as f64 / elapsed;
            }
        }
        0.0
    }
    
    /// 获取丢弃率
    pub fn get_drop_rate(&self) -> f64 {
        if self.messages_received > 0 {
            self.messages_dropped as f64 / self.messages_received as f64
        } else {
            0.0
        }
    }
}

/// 采样配置
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// 各服务类型的采样率 (0.0-1.0)
    pub rates: HashMap<String, f32>,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        let mut rates = HashMap::new();
        
        // 关键消息100%处理
        rates.insert("tracking".to_string(), 1.0);
        rates.insert("route".to_string(), 1.0);
        rates.insert("error_info".to_string(), 1.0);
        rates.insert("vcc".to_string(), 1.0);
        rates.insert("uos_config".to_string(), 1.0);
        
        // 背景消息采样处理
        rates.insert("traj".to_string(), 0.1);        // 10%
        rates.insert("moving_obj".to_string(), 0.05); // 5%
        rates.insert("device".to_string(), 0.2);      // 20%
        rates.insert("loc_stat".to_string(), 0.3);    // 30%
        
        Self { rates }
    }
}

impl SamplingConfig {
    /// 获取服务的采样率
    pub fn get_rate(&self, service: &str) -> f32 {
        self.rates.get(service).copied().unwrap_or(1.0)
    }
    
    /// 设置服务的采样率
    pub fn set_rate(&mut self, service: &str, rate: f32) {
        let rate = rate.clamp(0.0, 1.0); // 确保在有效范围内
        self.rates.insert(service.to_string(), rate);
    }
    
    /// 检查是否应该处理该消息
    pub fn should_process(&self, service: &str) -> bool {
        let rate = self.get_rate(service);
        
        if rate >= 1.0 {
            return true;
        }
        
        // 使用快速随机数生成
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        service.hash(&mut hasher);
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
            
        let random_val = (hasher.finish() % 1000) as f32 / 1000.0;
        random_val < rate
    }
}
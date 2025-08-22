#[cfg(test)]
mod tests {
    // use super::*;
    use crate::types::*;
    // use std::collections::HashMap;
    
    #[test]
    fn test_vehicle_message_creation() {
        let mut msg = VehicleMessage::new(
            "tracking".to_string(),
            "TEST_VIN_123".to_string(),
            1234567890.0
        );
        
        msg.channel = "tracking".to_string();
        msg.params.insert("data".to_string(), serde_json::json!({"x": 1.0, "y": 2.0}));
        
        assert!(msg.is_valid());
        assert_eq!(msg.service, "tracking");
        assert_eq!(msg.vin, "TEST_VIN_123");
        
        let hash1 = msg.get_hash();
        let hash2 = msg.get_hash();
        assert_eq!(hash1, hash2); // 相同消息应该有相同hash
    }
    
    #[test]
    fn test_message_priority() {
        assert_eq!(MessagePriority::from_service("tracking"), MessagePriority::Critical);
        assert_eq!(MessagePriority::from_service("traj"), MessagePriority::Background);
        assert_eq!(MessagePriority::from_service("vcc"), MessagePriority::Normal);
        
        assert!(MessagePriority::Critical.queue_capacity() > 0);
        assert!(MessagePriority::Critical.processing_interval().as_micros() > 0);
    }
    
    #[test]
    fn test_sampling_config() {
        let mut config = SamplingConfig::default();
        
        assert_eq!(config.get_rate("tracking"), 1.0);
        assert_eq!(config.get_rate("traj"), 0.1);
        
        config.set_rate("custom_service", 0.5);
        assert_eq!(config.get_rate("custom_service"), 0.5);
        
        // 测试边界值
        config.set_rate("test", 1.5); // 应该被限制为1.0
        assert_eq!(config.get_rate("test"), 1.0);
        
        config.set_rate("test", -0.5); // 应该被限制为0.0
        assert_eq!(config.get_rate("test"), 0.0);
    }
    
    #[test]
    fn test_processing_stats() {
        let mut stats = ProcessingStats::new();
        
        stats.increment_received();
        stats.increment_processed();
        stats.update_processing_time(std::time::Duration::from_micros(1000));
        stats.update_queue_size(50);
        
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.messages_processed, 1);
        assert_eq!(stats.avg_processing_time_us, 1000);
        assert_eq!(stats.queue_size, 50);
        assert_eq!(stats.get_drop_rate(), 0.0);
    }
    
    #[test]
    fn test_json_serialization() {
        let mut msg = VehicleMessage::new(
            "test".to_string(),
            "VIN123".to_string(),
            1234567890.0
        );
        
        msg.params.insert("test_param".to_string(), serde_json::json!({"value": 42}));
        
        // 测试序列化
        let json_str = serde_json::to_string(&msg).unwrap();
        assert!(json_str.contains("test"));
        assert!(json_str.contains("VIN123"));
        
        // 测试反序列化
        let deserialized: VehicleMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.service, msg.service);
        assert_eq!(deserialized.vin, msg.vin);
        assert_eq!(deserialized.timestamp, msg.timestamp);
    }
}
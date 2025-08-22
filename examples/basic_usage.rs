use vehicle_nn_core::*;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting Vehicle NN Core example");
    
    // 创建性能监控器
    let monitor = PerformanceMonitor::new(std::time::Duration::from_secs(5));
    
    // 创建采样配置
    let mut sampling_config = SamplingConfig::default();
    sampling_config.set_rate("example_service", 0.8);
    
    // 模拟消息处理
    for i in 0..1000 {
        let start_time = std::time::Instant::now();
        
        // 创建测试消息
        let mut message = VehicleMessage::new(
            if i % 10 == 0 { "tracking" } else { "traj" }.to_string(),
            format!("VIN_{}", i % 5),
            chrono::Utc::now().timestamp() as f64,
        );
        
        message.params.insert(
            "data".to_string(), 
            serde_json::json!({
                "x": i as f64 * 0.1,
                "y": i as f64 * 0.2,
                "speed": 30.0 + (i % 20) as f64
            })
        );
        
        // 检查是否应该处理
        if sampling_config.should_process(&message.service) {
            // 模拟消息处理
            process_message(&message).await?;
            
            let processing_time = start_time.elapsed();
            monitor.record_processed(processing_time);
        } else {
            monitor.record_dropped("sampling");
        }
        
        monitor.record_received();
        
        // 模拟处理间隔
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    
    // 输出最终统计
    let stats = monitor.get_stats();
    info!("Final statistics:");
    info!("  Received: {}", stats.messages_received);
    info!("  Processed: {}", stats.messages_processed);
    info!("  Dropped: {}", stats.messages_dropped);
    info!("  Drop rate: {:.2}%", stats.get_drop_rate() * 100.0);
    info!("  Avg processing time: {}μs", stats.avg_processing_time_us);
    info!("  Health status: {}", monitor.get_health_status().as_str());
    
    Ok(())
}

async fn process_message(message: &VehicleMessage) -> Result<()> {
    // 模拟不同类型消息的处理时间
    let processing_time = match message.service.as_str() {
        "tracking" => std::time::Duration::from_micros(500),  // 快速处理
        "traj" => std::time::Duration::from_micros(100),      // 很快处理
        _ => std::time::Duration::from_micros(300),           // 中等处理
    };
    
    tokio::time::sleep(processing_time).await;
    
    // 模拟处理逻辑
    if message.is_valid() {
        let priority = MessagePriority::from_service(&message.service);
        tracing::debug!(
            "Processed {} message from {} with priority {:?}",
            message.service,
            message.vin,
            priority
        );
    }
    
    Ok(())
}
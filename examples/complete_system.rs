use vehicle_nn_core::*;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, Level};
use tracing_subscriber;

/// 完整的车辆消息处理系统示例
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚗 Starting Vehicle NN Core Complete System Example");
    info!("📋 {}", get_library_info());

    // 1. 创建消息处理器
    let mut message_processor = MessageProcessor::new();
    
    // 设置消息处理回调
    message_processor.set_callback(Arc::new(|message| {
        handle_vehicle_message(message)
    }));
    
    let processor_arc = Arc::new(message_processor);
    
    // 2. 创建Nanomsg客户端配置
    let nanomsg_config = NanomsgConfig {
        listen_url: "ipc:///tmp/vehicle_test.ipc".to_string(),
        receive_timeout: Duration::from_millis(100),
        reconnect_interval: Duration::from_secs(2),
        max_reconnect_attempts: 5,
        buffer_size: 4096,
        batch_size: 50,
        batch_timeout: Duration::from_millis(5),
    };
    
    // 3. 创建Nanomsg客户端
    let nanomsg_client = NanomsgClient::new(nanomsg_config, processor_arc.clone());
    
    info!("🔧 System components initialized");
    
    // 4. 启动系统组件
    let processor_handle = {
        let processor = processor_arc.clone();
        tokio::spawn(async move {
            info!("🚀 Starting message processor...");
            if let Err(e) = processor.start().await {
                warn!("Message processor error: {}", e);
            }
        })
    };
    
    let client_handle = tokio::spawn(async move {
        info!("🌐 Starting Nanomsg client...");
        if let Err(e) = nanomsg_client.start().await {
            warn!("Nanomsg client error: {}", e);
        }
    });
    
    // 5. 启动监控任务
    let monitor_handle = {
        let processor = processor_arc.clone();
        tokio::spawn(async move {
            monitor_system_performance(processor).await;
        })
    };
    
    // 6. 模拟运行一段时间
    info!("⏱️  System running for 30 seconds...");
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // 7. 优雅关闭
    info!("🛑 Shutting down system...");
    
    // 停止处理器
    processor_arc.stop();
    
    // 等待任务完成或超时
    tokio::select! {
        _ = processor_handle => info!("✅ Message processor stopped"),
        _ = client_handle => info!("✅ Nanomsg client stopped"),
        _ = monitor_handle => info!("✅ Monitor stopped"),
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            warn!("⚠️  Shutdown timeout, forcing exit");
        }
    }
    
    // 8. 输出最终统计
    let final_stats = processor_arc.get_stats();
    print_final_statistics(&final_stats);
    
    info!("🏁 Vehicle NN Core system example completed");
    Ok(())
}

/// 处理车辆消息的回调函数
fn handle_vehicle_message(message: VehicleMessage) -> Result<()> {
    let priority = MessagePriority::from_service(&message.service);
    
    match message.service.as_str() {
        "tracking" => handle_tracking_message(&message)?,
        "route" => handle_route_message(&message)?,
        "error_info" => handle_error_message(&message)?,
        "traj" => handle_trajectory_message(&message)?,
        "moving_obj" => handle_moving_object_message(&message)?,
        "vcc" => handle_vcc_message(&message)?,
        "device" => handle_device_message(&message)?,
        _ => handle_unknown_message(&message)?,
    }
    
    // 记录处理日志（仅对关键消息）
    if priority == MessagePriority::Critical {
        info!(
            "🎯 Processed {} message from {} at {:.3}s",
            message.service,
            message.vin,
            message.timestamp
        );
    }
    
    Ok(())
}

/// 处理跟踪消息
fn handle_tracking_message(message: &VehicleMessage) -> Result<()> {
    if let Some(data) = message.params.get("data") {
        if let Some(obj) = data.as_object() {
            let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let speed = obj.get("speed").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            // 模拟跟踪数据处理
            if speed > 50.0 {
                info!("🚨 High speed detected: {:.1} km/h at ({:.2}, {:.2})", speed, x, y);
            }
        }
    }
    Ok(())
}

/// 处理路线消息
fn handle_route_message(message: &VehicleMessage) -> Result<()> {
    info!("🗺️  Processing route message for vehicle: {}", message.vin);
    // 模拟路线处理逻辑
    tokio::task::block_in_place(|| {
        std::thread::sleep(Duration::from_micros(500)); // 模拟处理时间
    });
    Ok(())
}

/// 处理错误消息
fn handle_error_message(message: &VehicleMessage) -> Result<()> {
    warn!("⚠️  Error message received from vehicle: {}", message.vin);
    // 模拟错误处理逻辑
    Ok(())
}

/// 处理轨迹消息
fn handle_trajectory_message(message: &VehicleMessage) -> Result<()> {
    // 轨迹消息处理很快
    tokio::task::block_in_place(|| {
        std::thread::sleep(Duration::from_micros(100));
    });
    Ok(())
}

/// 处理移动对象消息
fn handle_moving_object_message(message: &VehicleMessage) -> Result<()> {
    // 移动对象消息处理
    tokio::task::block_in_place(|| {
        std::thread::sleep(Duration::from_micros(200));
    });
    Ok(())
}

/// 处理VCC消息
fn handle_vcc_message(message: &VehicleMessage) -> Result<()> {
    info!("🔧 VCC message processed for vehicle: {}", message.vin);
    Ok(())
}

/// 处理设备消息
fn handle_device_message(message: &VehicleMessage) -> Result<()> {
    // 设备消息处理
    Ok(())
}

/// 处理未知消息
fn handle_unknown_message(message: &VehicleMessage) -> Result<()> {
    warn!("❓ Unknown message type: {} from vehicle: {}", message.service, message.vin);
    Ok(())
}

/// 系统性能监控任务
async fn monitor_system_performance(processor: Arc<MessageProcessor>) {
    info!("📊 Starting system performance monitor");
    
    let mut last_stats = processor.get_stats();
    let mut report_count = 0;
    
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        if !processor.is_running() {
            break;
        }
        
        let current_stats = processor.get_stats();
        report_count += 1;
        
        // 计算增量统计
        let msg_delta = current_stats.messages_received - last_stats.messages_received;
        let processed_delta = current_stats.messages_processed - last_stats.messages_processed;
        let dropped_delta = current_stats.messages_dropped - last_stats.messages_dropped;
        
        // 计算速率 (消息/秒)
        let receive_rate = msg_delta as f64 / 5.0;
        let process_rate = processed_delta as f64 / 5.0;
        
        info!(
            "📈 Performance Report #{} - Receive: {:.1}/s, Process: {:.1}/s, \
             Dropped: {}, Queue: {}, Avg Time: {}μs",
            report_count,
            receive_rate,
            process_rate,
            dropped_delta,
            current_stats.queue_size,
            current_stats.avg_processing_time_us
        );
        
        // 性能警告检查
        if current_stats.avg_processing_time_us > 5000 {
            warn!("🐌 High processing latency detected: {}μs", current_stats.avg_processing_time_us);
        }
        
        if current_stats.queue_size > 300 {
            warn!("📦 Large queue size detected: {}", current_stats.queue_size);
        }
        
        if dropped_delta > 10 {
            warn!("💧 High drop rate detected: {} messages in 5s", dropped_delta);
        }
        
        last_stats = current_stats;
    }
    
    info!("📊 Performance monitor stopped");
}

/// 打印最终统计信息
fn print_final_statistics(stats: &ProcessingStats) {
    println!("\n🏆 Final System Statistics:");
    println!("═══════════════════════════════════════");
    println!("📨 Messages Received:    {:>10}", stats.messages_received);
    println!("✅ Messages Processed:   {:>10}", stats.messages_processed);
    println!("💧 Messages Dropped:     {:>10}", stats.messages_dropped);
    println!("📊 Drop Rate:            {:>9.2}%", stats.get_drop_rate() * 100.0);
    println!("⚡ Avg Processing Time:  {:>7}μs", stats.avg_processing_time_us);
    println!("🚀 Processing Rate:      {:>7.1}/s", stats.get_processing_rate());
    println!("📦 Final Queue Size:     {:>10}", stats.queue_size);
    
    // 性能评级
    let performance_grade = if stats.get_drop_rate() < 0.01 && stats.avg_processing_time_us < 1000 {
        "🥇 Excellent"
    } else if stats.get_drop_rate() < 0.05 && stats.avg_processing_time_us < 5000 {
        "🥈 Good"
    } else if stats.get_drop_rate() < 0.1 && stats.avg_processing_time_us < 10000 {
        "🥉 Fair"
    } else {
        "⚠️  Needs Improvement"
    };
    
    println!("🏅 Performance Grade:    {}", performance_grade);
    println!("═══════════════════════════════════════\n");
}
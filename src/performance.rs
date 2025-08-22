use crate::types::ProcessingStats;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{info, warn};

/// 性能监控器
pub struct PerformanceMonitor {
    stats: Arc<RwLock<ProcessingStats>>,
    last_report_time: Arc<RwLock<Instant>>,
    report_interval: Duration,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(report_interval: Duration) -> Self {
        Self {
            stats: Arc::new(RwLock::new(ProcessingStats::new())),
            last_report_time: Arc::new(RwLock::new(Instant::now())),
            report_interval,
        }
    }
    
    /// 获取统计信息的只读引用
    pub fn get_stats(&self) -> ProcessingStats {
        self.stats.read().clone()
    }
    
    /// 记录接收到的消息
    pub fn record_received(&self) {
        let mut stats = self.stats.write();
        stats.increment_received();
        
        // 检查是否需要报告
        self.check_and_report();
    }
    
    /// 记录处理完成的消息
    pub fn record_processed(&self, processing_time: Duration) {
        let mut stats = self.stats.write();
        stats.increment_processed();
        stats.update_processing_time(processing_time);
        
        // 如果处理时间过长，记录警告
        if processing_time > Duration::from_millis(10) {
            warn!(
                "Slow message processing detected: {:.2}ms", 
                processing_time.as_secs_f64() * 1000.0
            );
        }
    }
    
    /// 记录丢弃的消息
    pub fn record_dropped(&self, reason: &str) {
        let mut stats = self.stats.write();
        stats.increment_dropped();
        
        warn!("Message dropped: {}", reason);
    }
    
    /// 更新队列大小
    pub fn update_queue_size(&self, size: usize) {
        let mut stats = self.stats.write();
        stats.update_queue_size(size);
        
        // 如果队列过大，记录警告
        if size > 800 {
            warn!("Large queue size detected: {}", size);
        }
    }
    
    /// 检查并报告性能统计
    fn check_and_report(&self) {
        let mut last_report = self.last_report_time.write();
        let now = Instant::now();
        
        if now.duration_since(*last_report) >= self.report_interval {
            let stats = self.stats.read();
            
            info!(
                "Performance Report - Received: {}, Processed: {}, Dropped: {}, \
                 Drop Rate: {:.2}%, Avg Processing Time: {}μs, Queue Size: {}, \
                 Processing Rate: {:.1} msg/s",
                stats.messages_received,
                stats.messages_processed,
                stats.messages_dropped,
                stats.get_drop_rate() * 100.0,
                stats.avg_processing_time_us,
                stats.queue_size,
                stats.get_processing_rate()
            );
            
            // 检查性能警告
            self.check_performance_warnings(&stats);
            
            *last_report = now;
        }
    }
    
    /// 检查性能警告
    fn check_performance_warnings(&self, stats: &ProcessingStats) {
        // 检查丢弃率
        if stats.get_drop_rate() > 0.05 {  // 5%
            warn!(
                "High drop rate detected: {:.2}%", 
                stats.get_drop_rate() * 100.0
            );
        }
        
        // 检查平均处理时间
        if stats.avg_processing_time_us > 5000 {  // 5ms
            warn!(
                "High average processing time: {}μs", 
                stats.avg_processing_time_us
            );
        }
        
        // 检查队列积压
        if stats.queue_size > 500 {
            warn!("Large queue backlog: {}", stats.queue_size);
        }
        
        // 检查处理速率
        let processing_rate = stats.get_processing_rate();
        if processing_rate < 100.0 && stats.messages_received > 1000 {
            warn!(
                "Low processing rate: {:.1} msg/s", 
                processing_rate
            );
        }
    }
    
    /// 重置统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = ProcessingStats::new();
        
        let mut last_report = self.last_report_time.write();
        *last_report = Instant::now();
        
        info!("Performance statistics reset");
    }
    
    /// 获取性能健康状态
    pub fn get_health_status(&self) -> HealthStatus {
        let stats = self.stats.read();
        
        let drop_rate = stats.get_drop_rate();
        let avg_time_ms = stats.avg_processing_time_us as f64 / 1000.0;
        let queue_size = stats.queue_size;
        
        if drop_rate > 0.1 || avg_time_ms > 10.0 || queue_size > 800 {
            HealthStatus::Critical
        } else if drop_rate > 0.05 || avg_time_ms > 5.0 || queue_size > 500 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

/// 健康状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl HealthStatus {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Warning => "warning", 
            HealthStatus::Critical => "critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(Duration::from_secs(1));
        
        // 模拟消息处理
        monitor.record_received();
        monitor.record_processed(Duration::from_micros(500));
        monitor.update_queue_size(10);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.messages_processed, 1);
        assert_eq!(stats.queue_size, 10);
        assert!(stats.avg_processing_time_us > 0);
    }
    
    #[test]
    fn test_health_status() {
        let monitor = PerformanceMonitor::new(Duration::from_secs(1));
        
        // 初始状态应该是健康的
        assert_eq!(monitor.get_health_status(), HealthStatus::Healthy);
        
        // 模拟高延迟 - 需要更高的延迟才能触发Critical状态
        for _ in 0..20 {
            monitor.record_processed(Duration::from_millis(25)); // 25ms > 10ms阈值
        }
        
        // 应该变为Critical状态
        assert_eq!(monitor.get_health_status(), HealthStatus::Critical);
    }
}
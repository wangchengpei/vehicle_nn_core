use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vehicle_nn_core::*;
use std::collections::HashMap;

fn create_test_message(service: &str, size: usize) -> VehicleMessage {
    let mut message = VehicleMessage::new(
        service.to_string(),
        "BENCH_VIN_123".to_string(),
        1234567890.0,
    );
    
    // 创建不同大小的测试数据
    let mut data = HashMap::new();
    for i in 0..size {
        data.insert(format!("key_{}", i), serde_json::json!(format!("value_{}", i)));
    }
    
    message.params.insert("data".to_string(), serde_json::json!(data));
    message
}

fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_message", size),
            size,
            |b, &size| {
                b.iter(|| {
                    black_box(create_test_message("tracking", size))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");
    
    for size in [1, 10, 100, 1000].iter() {
        let message = create_test_message("tracking", *size);
        
        group.bench_with_input(
            BenchmarkId::new("serialize", size),
            &message,
            |b, message| {
                b.iter(|| {
                    black_box(serde_json::to_string(message).unwrap())
                })
            },
        );
        
        let json_str = serde_json::to_string(&message).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize", size),
            &json_str,
            |b, json_str| {
                b.iter(|| {
                    black_box(serde_json::from_str::<VehicleMessage>(json_str).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_message_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_hash");
    
    for size in [1, 10, 100, 1000].iter() {
        let message = create_test_message("tracking", *size);
        
        group.bench_with_input(
            BenchmarkId::new("hash_calculation", size),
            &message,
            |b, message| {
                b.iter(|| {
                    black_box(message.get_hash())
                })
            },
        );
    }
    
    group.finish();
}

fn bench_sampling_decision(c: &mut Criterion) {
    let mut group = c.benchmark_group("sampling_decision");
    
    let config = SamplingConfig::default();
    let services = ["tracking", "traj", "moving_obj", "device"];
    
    for service in services.iter() {
        group.bench_with_input(
            BenchmarkId::new("should_process", service),
            service,
            |b, &service| {
                b.iter(|| {
                    black_box(config.should_process(service))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_priority_determination(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_determination");
    
    let services = ["tracking", "route", "traj", "moving_obj", "vcc", "error_info"];
    
    for service in services.iter() {
        group.bench_with_input(
            BenchmarkId::new("from_service", service),
            service,
            |b, &service| {
                b.iter(|| {
                    black_box(MessagePriority::from_service(service))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_message_serialization,
    bench_message_hash,
    bench_sampling_decision,
    bench_priority_determination
);
criterion_main!(benches);
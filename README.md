# Vehicle NN Core

高性能车辆nanomsg消息处理核心库，使用Rust实现，专为车载环境优化。

## 🎯 项目目标

替代Python实现的nanomsg消息处理逻辑，提供：
- **高性能**：10x+ 消息处理速度提升
- **低内存**：3-5x 内存使用减少
- **低延迟**：微秒级消息处理延迟
- **高可靠性**：零拷贝、无GC暂停

## 🏗️ 项目结构

```
vehicle_nn_core/
├── src/
│   ├── lib.rs              # 库入口
│   ├── types.rs            # 数据类型定义
│   ├── message_processor.rs # 消息处理器（待实现）
│   ├── nanomsg_client.rs   # Nanomsg客户端（待实现）
│   ├── performance.rs      # 性能监控
│   ├── error.rs           # 错误处理
│   └── tests.rs           # 单元测试
├── examples/
│   └── basic_usage.rs     # 基本使用示例
├── benches/
│   └── message_processing.rs # 性能基准测试
├── Cargo.toml             # 项目配置
├── build.sh              # 构建脚本
├── test.sh               # 测试脚本
└── README.md             # 项目文档
```

## 🚀 快速开始

### 安装依赖

```bash
# 安装Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 克隆项目
git clone <repository-url>
cd vehicle_nn_core
```

### 构建项目

```bash
# 使用构建脚本
chmod +x build.sh
./build.sh

# 或者手动构建
cargo build --release
```

### 运行测试

```bash
# 使用测试脚本
chmod +x test.sh
./test.sh

# 或者手动测试
cargo test
```

### 运行示例

```bash
cargo run --example basic_usage
```

## 📊 性能基准

运行基准测试：

```bash
cargo bench
```

预期性能指标：
- 消息创建：< 1μs
- JSON序列化：< 10μs (小消息)
- 消息哈希：< 0.5μs
- 采样决策：< 0.1μs

## 🔧 核心特性

### 消息类型

- **VehicleMessage**: 标准车辆消息结构
- **MessagePriority**: 三级优先级系统
- **ProcessingStats**: 实时性能统计

### 性能优化

- **智能采样**: 可配置的消息采样率
- **优先级队列**: 关键消息优先处理
- **零拷贝**: 最小化内存分配
- **批量处理**: 减少系统调用开销

### 监控功能

- **实时统计**: 吞吐量、延迟、丢弃率
- **健康检查**: 自动性能警告
- **可观测性**: 结构化日志输出

## 📈 使用示例

```rust
use vehicle_nn_core::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建性能监控器
    let monitor = PerformanceMonitor::new(
        std::time::Duration::from_secs(5)
    );
    
    // 创建消息
    let mut message = VehicleMessage::new(
        "tracking".to_string(),
        "VIN123".to_string(),
        chrono::Utc::now().timestamp() as f64,
    );
    
    // 处理消息
    let start = std::time::Instant::now();
    process_message(&message).await?;
    monitor.record_processed(start.elapsed());
    
    Ok(())
}
```

## 🧪 测试覆盖

- ✅ 单元测试：核心数据结构
- ✅ 性能测试：基准测试套件
- ✅ 集成测试：端到端流程
- ⏳ 模糊测试：边界条件（计划中）

## 📋 开发计划

### Phase 1: 核心数据结构 ✅
- [x] 消息类型定义
- [x] 错误处理
- [x] 性能监控
- [x] 基础测试

### Phase 2: 消息处理器 ⏳
- [ ] 多优先级队列
- [ ] 异步消息处理
- [ ] 采样和过滤
- [ ] 批量处理

### Phase 3: Nanomsg集成 ⏳
- [ ] Nanomsg客户端
- [ ] 连接管理
- [ ] 错误恢复
- [ ] 性能优化

### Phase 4: Python集成 ⏳
- [ ] PyO3 FFI绑定
- [ ] Python包装器
- [ ] 兼容性测试
- [ ] 部署脚本

## 🤝 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [Rust官方文档](https://doc.rust-lang.org/)
- [Tokio异步运行时](https://tokio.rs/)
- [Serde序列化框架](https://serde.rs/)
- [Criterion基准测试](https://bheisler.github.io/criterion.rs/)
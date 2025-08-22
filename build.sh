#!/bin/bash

# Vehicle NN Core 构建脚本

set -e

echo "🚗 Building Vehicle NN Core..."

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first."
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 显示Rust版本
echo "📋 Rust version:"
rustc --version
cargo --version

# 格式化代码
echo "🎨 Formatting code..."
cargo fmt

# 代码检查
echo "🔍 Running clippy..."
cargo clippy -- -D warnings

# 运行测试
echo "🧪 Running tests..."
cargo test --verbose

# 构建release版本
echo "🔨 Building release version..."
cargo build --release

# 运行基准测试
echo "📊 Running benchmarks..."
cargo bench

# 运行示例
echo "🚀 Running example..."
cargo run --example basic_usage

echo "✅ Build completed successfully!"
echo ""
echo "📁 Generated files:"
echo "   - target/release/libvehicle_nn_core.rlib (Rust library)"
echo "   - target/release/libvehicle_nn_core.so (Dynamic library)"
echo ""
echo "🎯 Next steps:"
echo "   1. Review benchmark results in target/criterion/"
echo "   2. Check test coverage with 'cargo tarpaulin'"
echo "   3. Generate documentation with 'cargo doc --open'"
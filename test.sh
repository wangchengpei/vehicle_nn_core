#!/bin/bash

# Vehicle NN Core 测试脚本

set -e

echo "🧪 Running Vehicle NN Core tests..."

# 单元测试
echo "📝 Running unit tests..."
cargo test --lib --verbose

# 集成测试
echo "🔗 Running integration tests..."
cargo test --tests --verbose

# 文档测试
echo "📚 Running doc tests..."
cargo test --doc --verbose

# 示例测试
echo "🚀 Testing examples..."
cargo run --example basic_usage

# 性能测试
echo "📊 Running performance benchmarks..."
cargo bench --bench message_processing

# 内存泄漏检查 (如果安装了valgrind)
if command -v valgrind &> /dev/null; then
    echo "🔍 Running memory leak check..."
    cargo build --example basic_usage
    valgrind --leak-check=full --show-leak-kinds=all \
        ./target/debug/examples/basic_usage
fi

# 代码覆盖率 (如果安装了tarpaulin)
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📈 Generating code coverage report..."
    cargo tarpaulin --out Html --output-dir coverage/
    echo "Coverage report generated in coverage/tarpaulin-report.html"
fi

echo "✅ All tests completed successfully!"
use nncp::{Socket, Protocol, Domain};
use anyhow::{Context, Result};
use tokio::task;
use tracing::{info, error};

/// Nanomsg PAIR 协议服务端
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let listen_url = "tcp://127.0.0.1:5555";
    info!("Starting Nanomsg PAIR server on: {}", listen_url);

    // 创建 PAIR 协议的 Socket（对应 Python 的 AF_SP + PAIR）
    let mut socket = Socket::new(Domain::SP, Protocol::Pair)
        .context("Failed to create socket")?;

    // 绑定地址（兼容 IPv4/IPv6）
    socket.bind(listen_url)
        .context("Failed to bind address")?;

    // 主循环：接收和处理消息
    let mut buffer = [0u8; 1024];
    loop {
        match socket.recv(&mut buffer) {
            Ok(bytes_received) => {
                let msg = String::from_utf8_lossy(&buffer[..bytes_received]);
                info!("Received message: {}", msg);

                // 示例：原样返回消息（PAIR 协议是双向通信）
                if let Err(e) = socket.send(&buffer[..bytes_received]) {
                    error!("Failed to send reply: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to receive message: {}", e);
                // 简单重连逻辑（生产环境应更健壮）
                if e.kind() == std::io::ErrorKind::ConnectionReset {
                    info!("Reconnecting...");
                    socket = Socket::new(Domain::SP, Protocol::Pair)?;
                    socket.bind(listen_url)?;
                }
            }
        }
    }
}
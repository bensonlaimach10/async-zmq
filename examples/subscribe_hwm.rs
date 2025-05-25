//! cargo run --example subscribe_hwm --features="rt-async-std"

use async_zmq::{Result, StreamExt};

#[async_std::main]
async fn main() -> Result<()> {
    // Create a subscriber with a receive high water mark of 1000 messages
    let socket = async_zmq::subscribe("tcp://127.0.0.1:5555")?;
    let mut subscriber = socket.connect()?;
    subscriber.set_receive_hwm(1000)?;

    // Subscribe to a topic
    subscriber.set_subscribe("topic")?;

    println!("Subscriber running with HWM: {}", subscriber.get_receive_hwm()?);
    println!("Waiting for messages...");

    while let Some(msg) = subscriber.next().await {
        let msg = msg?;
        if let Some(topic) = msg.first().and_then(|m| m.as_str()) {
            if let Some(content) = msg.get(1).and_then(|m| m.as_str()) {
                println!("Received on topic '{}': {}", topic, content);
            }
        }
    }

    Ok(())
} 
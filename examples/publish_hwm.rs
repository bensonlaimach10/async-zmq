//! cargo run --example publish_hwm --features="rt-async-std"

use async_zmq::{Result, SinkExt};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<()> {
    // Create a publisher with a send high water mark of 1000 messages
    let publisher = async_zmq::publish("tcp://127.0.0.1:5555")?
        .bind()?
        .set_send_hwm(1000)?;

    println!("Publisher running with HWM: {}", publisher.get_send_hwm()?);
    println!("Press Ctrl+C to stop");

    let mut counter = 0;
    loop {
        // Send messages with a topic
        let topic = b"topic".to_vec();
        let message = format!("Message {}", counter);
        
        publisher.send(vec![topic, message.into_bytes()].into()).await?;
        counter += 1;
        
        async_std::task::sleep(Duration::from_millis(100)).await;
    }
} 
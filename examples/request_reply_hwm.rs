//! cargo run --example request_reply_hwm --features="rt-async-std"

use async_zmq::Result;
use async_std::task;

async fn run_server() -> Result<()> {
    let server = async_zmq::reply("tcp://127.0.0.1:5555")?.bind()?;

    server
        .set_receive_hwm(1000)?
        .set_send_hwm(1000)?;

    println!("Server running with HWM - Receive: {}, Send: {}", 
        server.get_receive_hwm()?, 
        server.get_send_hwm()?);

    while let Ok(msg) = server.recv().await {
        println!("Server received: {:?}", msg);
        // Echo the message back
        server.send(msg).await?;
    }

    Ok(())
}

async fn run_client() -> Result<()> {
    let client = async_zmq::request("tcp://127.0.0.1:5555")?.connect()?;

    client
        .set_receive_hwm(1000)?
        .set_send_hwm(1000)?;

    println!("Client running with HWM - Receive: {}, Send: {}", 
        client.get_receive_hwm()?, 
        client.get_send_hwm()?);

    for i in 0..5 {
        let message = format!("Hello {}", i);
        println!("Client sending: {}", message);
        
        client.send(vec![message.into_bytes()]).await?;
        let response = client.recv().await?;
        
        println!("Client received: {:?}", response);
        task::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

#[async_std::main]
async fn main() -> Result<()> {
    // Run server in a separate task
    task::spawn(run_server());
    
    // Wait a bit for server to start
    task::sleep(std::time::Duration::from_secs(1)).await;
    
    // Run client
    run_client().await
} 
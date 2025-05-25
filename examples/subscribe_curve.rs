use async_zmq::{Result, StreamExt, CurveKeyPair};

#[async_std::main]
async fn main() -> Result<()> {
    // First check if CURVE is supported
    if !zmq::has("curve").unwrap_or(false) {
        eprintln!("Error: CURVE security is not supported by the ZeroMQ library.");
        eprintln!("This may be because the library was not compiled with libsodium support.");
        eprintln!("You need to rebuild ZeroMQ with CURVE support to use this example.");
        std::process::exit(1);
    }

    println!("Starting subscriber...");

    // Generate CURVE key pair
    println!("Generating subscriber key pair...");
    let subscriber_pair = CurveKeyPair::new()?;
    println!("Subscriber public key: {:?}", subscriber_pair.public_key);

    // Create and configure subscriber
    println!("Creating subscriber socket...");
    let mut subscriber = async_zmq::subscribe("tcp://127.0.0.1:5555")?.connect()?;

    println!("Setting CURVE options for subscriber...");
    


    
    subscriber
        // This should be the actual publisher's public key - for a real application, you'd need to get this
        // from the publisher or configure it separately
        .set_curve_serverkey(&[0u8; 32])? // Replace with actual publisher public key
        // Set CURVE public key
        .set_curve_publickey(&subscriber_pair.public_key)?
        // Set CURVE secret key
        .set_curve_secretkey(&subscriber_pair.secret_key)?
        // Set subscribe topic
        .set_subscribe("topic")?;

    println!("Subscriber running. Press Ctrl+C to exit.");
    println!("Subscriber public key (share this with publisher): {:?}", subscriber_pair.public_key);

    // Receive messages
    while let Some(msg) = subscriber.next().await {
        match msg {
            Ok(msg) => {
                let topic = String::from_utf8_lossy(&msg[0]);
                let content = String::from_utf8_lossy(&msg[1]);
                println!("Received message on topic '{}': {}", topic, content);
            }
            Err(e) => println!("Error receiving message: {:?}", e),
        }
    }

    Ok(())
} 
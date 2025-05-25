use async_zmq::{Result, SinkExt, CurveKeyPair};

#[async_std::main]
async fn main() -> Result<()> {
    // First check if CURVE is supported
    if !zmq::has("curve").unwrap_or(false) {
        eprintln!("Error: CURVE security is not supported by the ZeroMQ library.");
        eprintln!("This may be because the library was not compiled with libsodium support.");
        eprintln!("You need to rebuild ZeroMQ with CURVE support to use this example.");
        std::process::exit(1);
    }

    println!("Starting publisher...");

    // Generate CURVE key pair
    println!("Generating publisher key pair...");
    let publisher_pair = CurveKeyPair::new()?;
    println!("Publisher public key: {:?}", publisher_pair.public_key);

    // Create and configure publisher
    let mut publisher = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;

    // Setting CURVE options for publisher...
    publisher
        .set_curve_server(true)?
        .set_curve_secretkey(&publisher_pair.secret_key)?
        .set_curve_publickey(&publisher_pair.public_key)?;

    println!("Publisher running. Press Ctrl+C to exit.");
    println!("Publisher public key (share this with subscribers): {:?}", publisher_pair.public_key);

    // Send messages periodically
    let mut counter = 0;
    loop {
        let topic = b"topic".to_vec();
        let message = format!("secure message {}", counter);
        println!("Sending message {}", counter);
        
        let parts = vec![
            topic, 
            message.into_bytes()
        ];
        
        publisher.send(parts.into()).await?;
        counter += 1;
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    }
} 
use async_zmq::{Result, SinkExt, Context, CurveKeyPair};

// ZAP authentication handler
fn zap_auth_handler() -> Result<()> {
    let ctx = Context::new();
    let zap = ctx.socket(zmq::REP)?;
    zap.bind("inproc://zeromq.zap.01")?;

    println!("ZAP authentication handler started...");

    loop {
        let request = zap.recv_multipart(0)?;
        println!("Received ZAP request: {:?}", request);

        // Verify the request format
        if request.len() < 6 {
            println!("Invalid request format");
            continue;
        }

        let version = String::from_utf8_lossy(&request[0]);
        let request_id = String::from_utf8_lossy(&request[1]);
        let domain = String::from_utf8_lossy(&request[2]);
        let address = String::from_utf8_lossy(&request[3]);
        let identity = String::from_utf8_lossy(&request[4]);
        let mechanism = String::from_utf8_lossy(&request[5]);

        println!("Version: {}", version);
        println!("Request ID: {}", request_id);
        println!("Domain: {}", domain);
        println!("Address: {}", address);
        println!("Identity: {}", identity);
        println!("Mechanism: {}", mechanism);

        // For CURVE authentication, we need the client's public key
        if mechanism == "CURVE" && request.len() >= 7 {
            let client_key = String::from_utf8_lossy(&request[6]);
            println!("Client key: {}", client_key);

            // In a real application, you would verify the client's public key
            // against a list of allowed keys. For this example, we'll accept all keys.
            let response = vec![
                "1.0".to_string(),
                request_id.to_string(),
                "200".to_string(),
                "OK".to_string(),
                "user".to_string(),
                "authenticated".to_string(),
            ];
            zap.send_multipart(&response, 0)?;
        } else {
            // Reject other mechanisms or invalid requests
            let response = vec![
                "1.0".to_string(),
                request_id.to_string(),
                "400".to_string(),
                "Invalid request".to_string(),
                "".to_string(),
                "".to_string(),
            ];
            zap.send_multipart(&response, 0)?;
        }
    }
}

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

    // Start ZAP authentication handler in a separate thread
    std::thread::spawn(|| {
        if let Err(e) = zap_auth_handler() {
            eprintln!("ZAP handler error: {}", e);
        }
    });

    // Generate CURVE key pair
    println!("Generating publisher key pair...");
    let publisher_pair = CurveKeyPair::new()?;
    println!("Publisher public key: {:?}", publisher_pair.public_key);

    // Create and configure publisher
    println!("Creating publisher socket...");
    let mut publisher = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;

    println!("Setting CURVE options for publisher...");
    publisher.set_curve_server(true)?;
    publisher.set_curve_secretkey(&publisher_pair.secret_key)?;
    publisher.set_curve_publickey(&publisher_pair.public_key)?;
    publisher.set_zap_domain("global")?;

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
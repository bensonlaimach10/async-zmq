use async_zmq::{Result, SinkExt, StreamExt};
use zmq::{Context, CurveKeyPair};

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
    // Start ZAP authentication handler in a separate thread
    std::thread::spawn(|| {
        if let Err(e) = zap_auth_handler() {
            eprintln!("ZAP handler error: {}", e);
        }
    });

    // Generate CURVE key pairs
    let publisher_pair = CurveKeyPair::new()?;
    let subscriber_pair = CurveKeyPair::new()?;

    println!("Publisher public key: {:?}", publisher_pair.public_key);
    println!("Subscriber public key: {:?}", subscriber_pair.public_key);

    // Create and configure publisher
    let mut publisher = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
    publisher.set_curve_server(true)?;
    publisher.set_curve_secretkey(&publisher_pair.secret_key)?;
    publisher.set_curve_publickey(&publisher_pair.public_key)?;
    publisher.set_zap_domain("global")?;

    // Create and configure subscriber
    let mut subscriber = async_zmq::subscribe("tcp://127.0.0.1:5555")?.connect()?;
    subscriber.set_curve_serverkey(&publisher_pair.public_key)?;
    subscriber.set_curve_publickey(&subscriber_pair.public_key)?;
    subscriber.set_curve_secretkey(&subscriber_pair.secret_key)?;
    subscriber.set_zap_domain("global")?;
    subscriber.set_subscribe("topic")?;

    // Send a message
    publisher.send(vec!["topic", "secure message"].into()).await?;
    println!("Message sent");

    // Receive the message
    if let Some(msg) = subscriber.next().await {
        match msg {
            Ok(msg) => println!("Received message: {:?}", msg.iter()),
            Err(e) => println!("Error receiving message: {:?}", e),
        }
    }

    Ok(())
} 
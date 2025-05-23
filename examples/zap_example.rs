use async_zmq::{Context, Socket, SocketType, Result, CurveKeyPair};
use std::env;
use std::thread;

// This example shows how to use ZeroMQ Authentication Protocol (ZAP)
// We create a ZAP handler that checks client credentials
// Then we create a secure server with CURVE that delegates authentication to ZAP
// Finally, we create a client that connects to the server

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = Context::new();

    // Generate server key pair
    let server_keys = CurveKeyPair::new().expect("Failed to generate CURVE keypair");
    
    println!("Generated server keypair:");
    println!("  Public key: {}", server_keys.public_key);
    println!("  Secret key: {}", server_keys.secret_key);

    // Run in server or client mode based on command-line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example zap_example [server|client]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(&ctx, &server_keys).await?,
        "client" => run_client(&ctx, &server_keys).await?,
        _ => {
            println!("Usage: cargo run --example zap_example [server|client]");
            std::process::exit(1);
        }
    }

    Ok(())
}

// Start a ZAP handler thread
fn start_zap_handler(ctx: &zmq::Context) -> thread::JoinHandle<()> {
    let zap_context = ctx.clone();
    thread::spawn(move || {
        // Create ZAP handler socket
        let zap = zap_context.socket(zmq::SocketType::REP).unwrap();
        zap.bind("inproc://zeromq.zap.01").unwrap();
        
        println!("ZAP handler started");
        
        loop {
            // Get all frames from the request
            let frames = zap.recv_multipart(0).unwrap();
            
            if frames.len() < 6 {
                println!("Invalid ZAP request, not enough frames");
                continue;
            }
            
            // Basic ZAP request structure:
            // Frame 0: Version (must be "1.0")
            // Frame 1: Request ID
            // Frame 2: Domain
            // Frame 3: Address
            // Frame 4: Identity
            // Frame 5: Mechanism (e.g., "CURVE")
            // Frame 6+: Credentials (depends on mechanism)
            
            let version = std::str::from_utf8(&frames[0]).unwrap_or("INVALID");
            let request_id = &frames[1];
            let domain = std::str::from_utf8(&frames[2]).unwrap_or("INVALID");
            let address = std::str::from_utf8(&frames[3]).unwrap_or("INVALID");
            let identity = hex::encode(&frames[4]);
            let mechanism = std::str::from_utf8(&frames[5]).unwrap_or("INVALID");
            
            println!("ZAP request received:");
            println!("  Version: {}", version);
            println!("  Domain: {}", domain);
            println!("  Address: {}", address);
            println!("  Identity: {}", identity);
            println!("  Mechanism: {}", mechanism);
            
            // For CURVE mechanism, extract client's public key
            if mechanism == "CURVE" && frames.len() >= 7 {
                let client_key = hex::encode(&frames[6]);
                println!("  Client public key: {}", client_key);
                
                // For this example, accept all clients
                // In a real app, you'd check the key against a whitelist or database
                
                // Send a successful response
                let reply = vec![
                    version.as_bytes().to_vec(),
                    request_id.to_vec(),
                    b"200".to_vec(),
                    b"OK".to_vec(),
                    b"".to_vec(),
                    b"".to_vec(),
                ];
                zap.send_multipart(reply, 0).unwrap();
                println!("ZAP authentication successful");
            } else {
                // Send a failure response
                let reply = vec![
                    version.as_bytes().to_vec(),
                    request_id.to_vec(),
                    b"400".to_vec(),
                    b"Invalid mechanism or credentials".to_vec(),
                    b"".to_vec(),
                    b"".to_vec(),
                ];
                zap.send_multipart(reply, 0).unwrap();
                println!("ZAP authentication failed");
            }
        }
    })
}

async fn run_server(ctx: &Context, keys: &CurveKeyPair) -> Result<()> {
    println!("Starting secure server with ZAP authentication...");
    
    // Start ZAP handler in a separate thread
    let zap_handler = start_zap_handler(&ctx.zmq_context());
    println!("ZAP handler started");
    
    // Create a socket
    let mut socket = ctx.socket(SocketType::REP)?;
    
    // Set ZAP domain - this activates authentication for this socket
    socket.set_zap_domain("global")?;
    
    // Set up CURVE security
    socket.set_curve_server(true)?;
    socket.set_curve_secretkey(&keys.secret_key)?;
    socket.set_curve_publickey(&keys.public_key)?;
    
    // Bind to an endpoint
    socket.bind("tcp://*:5555")?;
    
    println!("Server is running on tcp://*:5555");
    println!("Waiting for secure connection...");

    // Process messages
    loop {
        // Wait for a message
        let msg = socket.recv_msg().await?;
        let msg_str = msg.as_str().unwrap_or("<binary data>");
        println!("Received: {}", msg_str);
        
        // Send a reply
        socket.send("Hello from ZAP-authenticated server").await?;
    }
}

async fn run_client(ctx: &Context, server_keys: &CurveKeyPair) -> Result<()> {
    println!("Starting secure client...");
    
    // Generate client key pair
    let client_keys = CurveKeyPair::new().expect("Failed to generate CURVE keypair");
    
    println!("Generated client keypair:");
    println!("  Public key: {}", client_keys.public_key);
    println!("  Secret key: {}", client_keys.secret_key);
    
    // Create a socket
    let mut socket = ctx.socket(SocketType::REQ)?;
    
    // Set up CURVE security
    socket.set_curve_serverkey(&server_keys.public_key)?;
    socket.set_curve_secretkey(&client_keys.secret_key)?;
    socket.set_curve_publickey(&client_keys.public_key)?;
    
    // Connect to the server
    socket.connect("tcp://localhost:5555")?;
    
    println!("Connected to secure server");
    
    // Send a message
    socket.send("Hello from secure client with ZAP").await?;
    println!("Sent message to server");
    
    // Wait for a reply
    let reply = socket.recv_msg().await?;
    let reply_str = reply.as_str().unwrap_or("<binary data>");
    println!("Received reply: {}", reply_str);
    
    Ok(())
} 
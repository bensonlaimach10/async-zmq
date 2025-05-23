use async_zmq::{Context, Socket, SocketType, Result, CurveKeyPair};
use std::env;

// This example shows how to use ZeroMQ CURVE security.
// First, we generate a server key pair.
// Then we create a server that uses the private key to secure its endpoint.
// Finally, we create a client that uses the server's public key to connect securely.

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
        println!("Usage: cargo run --example curve_example [server|client]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(&ctx, &server_keys).await?,
        "client" => run_client(&ctx, &server_keys).await?,
        _ => {
            println!("Usage: cargo run --example curve_example [server|client]");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_server(ctx: &Context, keys: &CurveKeyPair) -> Result<()> {
    println!("Starting secure server...");
    
    // Create a socket
    let mut socket = ctx.socket(SocketType::REP)?;
    
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
        socket.send("Hello from secure server").await?;
    }
}

async fn run_client(ctx: &Context, server_keys: &CurveKeyPair) -> Result<()> {
    println!("Starting secure client...");
    
    // Generate client key pair
    let client_keys = CurveKeyPair::new().expect("Failed to generate CURVE keypair");
    
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
    socket.send("Hello from secure client").await?;
    println!("Sent message to server");
    
    // Wait for a reply
    let reply = socket.recv_msg().await?;
    let reply_str = reply.as_str().unwrap_or("<binary data>");
    println!("Received reply: {}", reply_str);
    
    Ok(())
} 
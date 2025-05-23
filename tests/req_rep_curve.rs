use std::time::Duration;
use std::thread;
use async_zmq::{Result, Context, CurveKeyPair, Message};

// Helper function to check if CURVE is supported
fn check_curve_support() -> bool {
    zmq::has("curve").unwrap_or(false)
}

// Test CURVE key pair generation for REQ-REP
#[test]
fn test_curve_key_pair_generation() -> Result<()> {
    if !check_curve_support() {
        println!("Skipping test: CURVE security not supported");
        return Ok(());
    }

    let pair = CurveKeyPair::new()?;
    
    // Check that keys are valid (32 bytes for Z85-encoded keys)
    assert_eq!(pair.public_key.len(), 32);
    assert_eq!(pair.secret_key.len(), 32);
    
    // Public and secret keys should be different
    assert_ne!(pair.public_key, pair.secret_key);
    
    Ok(())
}

// Test basic secure request-reply communication
#[async_std::test]
async fn test_secure_req_rep() -> Result<()> {
    if !check_curve_support() {
        println!("Skipping test: CURVE security not supported");
        return Ok(());
    }

    let ctx = Context::new();
    let uri = "tcp://127.0.0.1:5559";
    
    // Generate server (replier) key pair
    let server_pair = CurveKeyPair::new()?;
    
    // Generate client (requester) key pair
    let client_pair = CurveKeyPair::new()?;
    
    // Create and configure replier as CURVE server
    let replier = async_zmq::reply(uri)?.with_context(&ctx).bind()?;
    replier.set_curve_server(true)?;
    replier.set_curve_secretkey(&server_pair.secret_key)?;
    replier.set_curve_publickey(&server_pair.public_key)?;
    
    // Create and configure requester as CURVE client
    let requester = async_zmq::request(uri)?.with_context(&ctx).connect()?;
    requester.set_curve_serverkey(&server_pair.public_key)?;
    requester.set_curve_publickey(&client_pair.public_key)?;
    requester.set_curve_secretkey(&client_pair.secret_key)?;
    
    // Allow time for connection to establish
    async_std::task::sleep(Duration::from_millis(500)).await;
    
    // Create messages
    let request_message = Message::from("secure request message");
    let reply_message = Message::from("secure reply message");
    
    // We'll handle the request/reply in sequential order rather than with threads
    
    // Send request
    requester.send(vec![request_message]).await?;
    
    // Receive request at server
    let msg = replier.recv().await?;
    assert_eq!(msg.len(), 1);
    assert_eq!(msg[0].as_str().unwrap(), "secure request message");
    
    // Send reply
    replier.send(vec![reply_message]).await?;
    
    // Receive reply at client
    let reply = requester.recv().await?;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].as_str().unwrap(), "secure reply message");
    
    Ok(())
}

// Test client authentication failure with incorrect server key
#[async_std::test]
async fn test_incorrect_server_key() -> Result<()> {
    if !check_curve_support() {
        println!("Skipping test: CURVE security not supported");
        return Ok(());
    }
    
    let ctx = Context::new();
    let uri = "tcp://127.0.0.1:5560";
    
    // Generate server (replier) key pair
    let server_pair = CurveKeyPair::new()?;
    
    // Generate client (requester) key pair
    let client_pair = CurveKeyPair::new()?;
    
    // Generate a different server key pair (wrong keys)
    let wrong_server_pair = CurveKeyPair::new()?;
    
    // Create and configure replier as CURVE server
    let replier = async_zmq::reply(uri)?.with_context(&ctx).bind()?;
    replier.set_curve_server(true)?;
    replier.set_curve_secretkey(&server_pair.secret_key)?;
    replier.set_curve_publickey(&server_pair.public_key)?;
    
    // Create and configure requester with WRONG server key
    let requester = async_zmq::request(uri)?.with_context(&ctx).connect()?;
    requester.set_curve_serverkey(&wrong_server_pair.public_key)?; // Wrong key
    requester.set_curve_publickey(&client_pair.public_key)?;
    requester.set_curve_secretkey(&client_pair.secret_key)?;
    
    // Allow time for connection attempt
    async_std::task::sleep(Duration::from_millis(500)).await;
    
    // Send a request - this should fail to be delivered
    let request_message = Message::from("secure request message");
    
    // Send request
    requester.send(vec![request_message]).await?;
    
    // Set a timeout for receiving - we expect no message due to auth failure
    let timeout = async_std::future::timeout(
        Duration::from_millis(1000), 
        replier.recv()
    ).await;
    
    // We expect a timeout (no message) because authentication should fail
    assert!(timeout.is_err());
    
    Ok(())
}

// Test ZAP authentication with CURVE for REQ-REP
#[async_std::test]
async fn test_zap_authentication() -> Result<()> {
    if !check_curve_support() {
        println!("Skipping test: CURVE security not supported");
        return Ok(());
    }
    
    let ctx = Context::new();
    let uri = "tcp://127.0.0.1:5561";
    
    // Start ZAP handler in a separate thread
    let zap_ctx = ctx.clone();
    thread::spawn(move || -> Result<()> {
        let zap = zap_ctx.socket(zmq::REP)?;
        zap.bind("inproc://zeromq.zap.01")?;
        
        // Process one ZAP request
        let request = zap.recv_multipart(0)?;
        
        // Verify request format
        if request.len() >= 6 {
            let version = String::from_utf8_lossy(&request[0]);
            let request_id = &request[1];
            let mechanism = String::from_utf8_lossy(&request[5]);
            
            // Accept the authentication
            if mechanism == "CURVE" && version == "1.0" {
                let response = vec![
                    b"1.0".to_vec(),
                    request_id.to_vec(),
                    b"200".to_vec(),
                    b"OK".to_vec(),
                    b"".to_vec(),
                    b"".to_vec(),
                ];
                zap.send_multipart(&response, 0)?;
            }
        }
        
        Ok(())
    });
    
    // Generate server (replier) key pair
    let server_pair = CurveKeyPair::new()?;
    
    // Generate client (requester) key pair
    let client_pair = CurveKeyPair::new()?;
    
    // Create and configure replier as CURVE server with ZAP domain
    let replier = async_zmq::reply(uri)?.with_context(&ctx).bind()?;
    replier.set_curve_server(true)?;
    replier.set_curve_secretkey(&server_pair.secret_key)?;
    replier.set_curve_publickey(&server_pair.public_key)?;
    
    // Set ZAP domain using raw socket
    replier.as_raw_socket().set_zap_domain("global")?;
    
    // Create and configure requester as CURVE client
    let requester = async_zmq::request(uri)?.with_context(&ctx).connect()?;
    requester.set_curve_serverkey(&server_pair.public_key)?;
    requester.set_curve_publickey(&client_pair.public_key)?;
    requester.set_curve_secretkey(&client_pair.secret_key)?;
    
    // Allow time for connection and ZAP authentication
    async_std::task::sleep(Duration::from_millis(1000)).await;
    
    // Create messages
    let request_message = Message::from("secure request message");
    let reply_message = Message::from("secure reply message");
    
    // Sequential flow instead of threads
    
    // Send request
    requester.send(vec![request_message]).await?;
    
    // Receive request at server
    let msg = replier.recv().await?;
    assert_eq!(msg.len(), 1);
    assert_eq!(msg[0].as_str().unwrap(), "secure request message");
    
    // Send reply
    replier.send(vec![reply_message]).await?;
    
    // Receive reply at client
    let reply = requester.recv().await?;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].as_str().unwrap(), "secure reply message");
    
    Ok(())
}

// Test CURVE with custom context
#[async_std::test]
async fn test_custom_context() -> Result<()> {
    if !check_curve_support() {
        println!("Skipping test: CURVE security not supported");
        return Ok(());
    }
    
    let custom_ctx = Context::new();
    let uri = "tcp://127.0.0.1:5562";
    
    // Generate server (replier) key pair
    let server_pair = CurveKeyPair::new()?;
    
    // Generate client (requester) key pair
    let client_pair = CurveKeyPair::new()?;
    
    // Create and configure replier as CURVE server with custom context
    let replier = async_zmq::reply(uri)?.with_context(&custom_ctx).bind()?;
    replier.set_curve_server(true)?;
    replier.set_curve_secretkey(&server_pair.secret_key)?;
    replier.set_curve_publickey(&server_pair.public_key)?;
    
    // Create and configure requester as CURVE client with custom context
    let requester = async_zmq::request(uri)?.with_context(&custom_ctx).connect()?;
    requester.set_curve_serverkey(&server_pair.public_key)?;
    requester.set_curve_publickey(&client_pair.public_key)?;
    requester.set_curve_secretkey(&client_pair.secret_key)?;
    
    // Allow time for connection to establish
    async_std::task::sleep(Duration::from_millis(500)).await;
    
    // Create messages
    let request_message = Message::from("secure request message");
    let reply_message = Message::from("secure reply message");
    
    // Sequential flow instead of threads
    
    // Send request
    requester.send(vec![request_message]).await?;
    
    // Receive request at server
    let msg = replier.recv().await?;
    assert_eq!(msg.len(), 1);
    assert_eq!(msg[0].as_str().unwrap(), "secure request message");
    
    // Send reply
    replier.send(vec![reply_message]).await?;
    
    // Receive reply at client
    let reply = requester.recv().await?;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].as_str().unwrap(), "secure reply message");
    
    Ok(())
} 
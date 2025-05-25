use async_zmq::{Result, CurveKeyPair};

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting requester...");
    
    // Generate CURVE key pair for the requester
    println!("Generating requester key pair...");
    let requester_pair = CurveKeyPair::new()?;
    println!("Requester public key: {:?}", requester_pair.public_key);
    
    // Create a requester socket
    println!("Creating requester socket...");
    let requester = async_zmq::request("tcp://127.0.0.1:5555")?.connect()?;
    println!("Requester socket created and connected");

    // Set CURVE options for the requester
    requester
        // Set CURVE server flag
        .set_curve_server(true)?
        // Set CURVE secret key
        .set_curve_secretkey(&requester_pair.secret_key)?
        // Set CURVE public key"
        .set_curve_publickey(&requester_pair.public_key)?;

    // Send a request
    println!("Sending request...");
    requester.send(vec!["secure request"]).await?;
    println!("Request sent");

    // Receive the reply
    println!("Waiting for reply...");
    let reply = requester.recv().await?;
    println!("Received reply: {:?}", reply.iter());

    Ok(())
} 
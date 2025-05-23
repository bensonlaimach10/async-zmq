use async_zmq::{Result, CurveKeyPair};

#[async_std::main]
async fn main() -> Result<()> {
    println!("Starting replier...");
    
    // Generate CURVE key pair for the replier
    println!("Generating replier key pair...");
    let replier_pair = CurveKeyPair::new()?;
    println!("Replier public key: {:?}", replier_pair.public_key);
    
    // Create a replier socket
    println!("Creating replier socket...");
    let replier = async_zmq::reply("tcp://127.0.0.1:5555")?.bind()?;
    println!("Replier socket created and bound");

    // Set CURVE options for the replier
    println!("Setting CURVE options for replier...");
    replier.set_curve_server(true)?;
    println!("Set CURVE server flag");
    replier.set_curve_secretkey(&replier_pair.secret_key)?;
    println!("Set CURVE secret key");
    replier.set_curve_publickey(&replier_pair.public_key)?;
    println!("Set CURVE public key");

    // Receive the request and send a reply
    println!("Waiting for request...");
    let msg = replier.recv().await?;
    println!("Received request: {:?}", msg.iter());
    
    println!("Sending reply...");
    replier.send(vec!["secure reply"]).await?;
    println!("Reply sent");

    Ok(())
} 
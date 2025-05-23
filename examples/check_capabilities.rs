use async_zmq::Result;

fn main() -> Result<()> {
    println!("ZeroMQ version: {:?}", zmq::version());
    
    println!("Checking ZeroMQ capabilities:");
    println!("  CURVE: {}", zmq::has("curve").unwrap_or(false));
    println!("  PUB: {}", zmq::has("pub").unwrap_or(false));
    println!("  SUB: {}", zmq::has("sub").unwrap_or(false));
    println!("  REQ: {}", zmq::has("req").unwrap_or(false));
    println!("  REP: {}", zmq::has("rep").unwrap_or(false));
    println!("  DEALER: {}", zmq::has("dealer").unwrap_or(false));
    println!("  ROUTER: {}", zmq::has("router").unwrap_or(false));
    println!("  PULL: {}", zmq::has("pull").unwrap_or(false));
    println!("  PUSH: {}", zmq::has("push").unwrap_or(false));
    println!("  PAIR: {}", zmq::has("pair").unwrap_or(false));
    println!("  STREAM: {}", zmq::has("stream").unwrap_or(false));
    
    Ok(())
} 
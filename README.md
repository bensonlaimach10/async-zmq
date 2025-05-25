# Async ZeroMQ for Rust

An asynchronous Rust wrapper for ZeroMQ (libzmq) that provides a modern, ergonomic API with method chaining support.

## Features

- Asynchronous API built on top of async-std
- Method chaining for socket configuration
- Full support for ZeroMQ patterns (PUB-SUB, REQ-REP, etc.)
- CURVE security protocol integration
- Type-safe message handling

## Usage Examples

### Subscribe Socket Example

```rust
use async_zmq::{Result, StreamExt};

#[async_std::main]
async fn main() -> Result<()> {
    let mut zmq = async_zmq::subscribe("tcp://127.0.0.1:5555")?
        .connect()?
        .set_subscribe("topic")?;

    while let Some(msg) = zmq.next().await {
        let msg = msg?;
        println!("{:?}", msg.iter());
    }
    Ok(())
}
```

### Reply Socket with CURVE Security

```rust
use async_zmq::{Result, CurveKeyPair};

#[async_std::main]
async fn main() -> Result<()> {
    let replier_pair = CurveKeyPair::new()?;
    
    let replier = async_zmq::reply("tcp://127.0.0.1:5555")?
        .bind()?
        .set_curve_server(true)?
        .set_curve_secretkey(&replier_pair.secret_key)?
        .set_curve_publickey(&replier_pair.public_key)?;
        
    let msg = replier.recv().await?;
    replier.send(vec!["secure reply"]).await?;
    
    Ok(())
}
```

## Socket Types

The library supports all major ZeroMQ socket types with an ergonomic async API:

- PUB-SUB (Publisher-Subscriber)
- REQ-REP (Request-Reply)
- DEALER-ROUTER
- PUSH-PULL
- And more...

Each socket type provides specific methods relevant to its pattern while maintaining a consistent API style with method chaining support.

## Security

Built-in support for ZeroMQ security mechanisms:

- CURVE encryption
- ZAP authentication
- Domain configuration

All security options can be configured using method chaining:

```rust
socket
    .set_curve_server(true)?
    .set_curve_secretkey(&secret_key)?
    .set_curve_publickey(&public_key)?
    .set_zap_domain("global")?;
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
async-zmq = "0.1.0"
```

## Requirements

- Rust stable or nightly
- libzmq (zeromq) installed on your system
- async-std for async runtime

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed under the terms of both the Apache License, Version 2.0 and the MIT license without any additional terms or conditions.

## License

This project is licensed under both:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
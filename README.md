# Async ZeroMQ for Rust

An asynchronous Rust wrapper for ZeroMQ (libzmq) that provides a modern, ergonomic API with method chaining support.

## 🚀 Features

* Support for all ZeroMQ socket types.
* Stream and Sink interfaces for receiving and sending multipart messages.
* Compatible with any async runtime.
* CURVE encryption and ZAP authentication support.
* High Water Mark (HWM) control for message queuing.

---

## 🧰 Installation

Before using this crate, you must have the following system dependencies installed:

```bash
# On Debian/Ubuntu
sudo apt install libzmq3-dev libsodium-dev
```

If these libraries are not available, CURVE authentication will not work, and you will see runtime errors like:

```
Error: CURVE security is not supported by the ZeroMQ library.
```

Make sure to build ZeroMQ with **libsodium** support if you compile it manually.

---

## 📦 Usage

### Basic Publisher Example

```rust
let mut socket = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
socket.send(vec!["topic".into(), "message".into()].into()).await?;
```

### Shared Context Between Sockets

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

## Message Queue Control

All sockets support High Water Mark (HWM) settings to control message queuing behavior:

```rust
// Publisher with send HWM
let publisher = async_zmq::publish("tcp://127.0.0.1:5555")?
    .bind()?
    .set_send_hwm(1000)?;

// Subscriber with receive HWM
let subscriber = async_zmq::subscribe("tcp://127.0.0.1:5555")?
    .connect()?
    .set_receive_hwm(1000)?;

// Request-Reply with both send and receive HWM
let replier = async_zmq::reply("tcp://127.0.0.1:5555")?
    .bind()?
    .set_receive_hwm(1000)?
    .set_send_hwm(1000)?;
```

The High Water Mark is a hard limit on the maximum number of outstanding messages ØMQ shall queue in memory for any single peer. When this limit is reached:
- PUB sockets will drop messages for slow subscribers
- SUB sockets will drop messages if they can't process them fast enough
- REQ-REP sockets will block until space is available

Default HWM is 1000 messages. Setting it to 0 means "no limit".

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
# Async version for ZeroMQ bindings

[![][crates-badge]][crates-url] ![][license-badge] ![][build-badge]

[crates-badge]: https://img.shields.io/crates/v/async-zmq
[crates-url]: https://crates.io/crates/async_zmq
[license-badge]: https://img.shields.io/crates/l/async-zmq
[build-badge]: https://img.shields.io/github/actions/workflow/status/rdelfin/async-zmq/main.yml?branch=main

**`async-zmq`** is a high-level, runtime-agnostic asynchronous interface for the \[`zmq`] crate.
It seamlessly integrates with any async runtime (**tokio**, **async-std**, etc.), making ZeroMQ simple, safe, and ergonomic in async Rust.

---

## Features

* Support for all ZeroMQ socket types.
* Stream and Sink interfaces for receiving and sending multipart messages.
* Compatible with any async runtime.
* Optional support for CURVE authentication with ZAP handlers.

---

## Getting Started

### Basic Publisher Example

```rust
let mut socket = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
socket.send(vec!["topic".into(), "message".into()].into()).await?;
```

### Shared Context Between Sockets

```rust
use async_zmq::Context;

let ctx = Context::new();
let xpub = async_zmq::xpublish("inproc://example")?.with_context(&ctx).bind()?;
let sub = async_zmq::subscribe("inproc://example")?.with_context(&ctx).connect()?;
```

---

## CURVE Authentication

`async-zmq` supports CURVE encryption and authentication for secure communication. To use CURVE, both server and clients must have a key pair.

### Generating Keys

```rust
use async_zmq::CurveKeyPair;

let keys = CurveKeyPair::new()?;
println!("Public: {:?}", keys.public_key);
println!("Secret: {:?}", keys.secret_key);
```

### Setting CURVE Options for Server (Publisher)

```rust
let mut socket = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;

socket.set_curve_server(true)?;
socket.set_curve_secretkey(&keys.secret_key)?;
socket.set_curve_publickey(&keys.public_key)?;
socket.set_zap_domain("global")?;
```

---

## ZAP Authentication Handler

ZeroMQ's ZAP (ZMQ Authentication Protocol) is used to handle authentication requests when using CURVE. You must implement a simple in-process handler.

### Example ZAP Handler

```rust
fn zap_auth_handler() -> async_zmq::Result<()> {
    let ctx = async_zmq::Context::new();
    let zap = ctx.socket(zmq::REP)?;
    zap.bind("inproc://zeromq.zap.01")?;

    loop {
        let request = zap.recv_multipart(0)?;
        // Basic CURVE authentication validation
        if request.len() >= 7 && &request[5] == b"CURVE" {
            let client_key = String::from_utf8_lossy(&request[6]);
            println!("Client key: {}", client_key);

            // Accept all clients for this example
            let response = vec![
                "1.0", &request[1], "200", "OK", "user", "authenticated"
            ];
            zap.send_multipart(&response, 0)?;
        } else {
            // Reject invalid requests
            let response = vec![
                "1.0", &request[1], "400", "Invalid", "", ""
            ];
            zap.send_multipart(&response, 0)?;
        }
    }
}
```

> **Note:** You must run the ZAP handler in a separate thread before establishing CURVE sockets.

---

## Example: Secure Publisher

Here’s a simplified version of a secure publisher using CURVE and ZAP:

```rust
#[async_std::main]
async fn main() -> async_zmq::Result<()> {
    std::thread::spawn(|| {
        zap_auth_handler().unwrap();
    });

    let keys = CurveKeyPair::new()?;
    let mut pub_socket = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
    pub_socket.set_curve_server(true)?;
    pub_socket.set_curve_secretkey(&keys.secret_key)?;
    pub_socket.set_curve_publickey(&keys.public_key)?;
    pub_socket.set_zap_domain("global")?;

    let mut counter = 0;
    loop {
        let msg = format!("message {}", counter).into_bytes();
        pub_socket.send(vec![b"topic".to_vec(), msg].into()).await?;
        counter += 1;
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    }
}
```

---

## License

Licensed under [MIT](LICENSE).

---

## Links

* [ZeroMQ](https://zeromq.org)
* [zmq crate](https://crates.io/crates/zmq)
* [async-std](https://crates.io/crates/async-std)
* [tokio](https://crates.io/crates/tokio)

Let me know if you'd like this saved as a `README.md` file or further customized for `tokio` examples, client setup, or advanced ZAP handling.

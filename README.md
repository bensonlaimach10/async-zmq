# Async version for ZeroMQ bindings

[![][crates-badge]][crates-url] ![][license-badge] ![][build-badge]

[crates-badge]: https://img.shields.io/crates/v/async-zmq
[crates-url]: https://crates.io/crates/async_zmq
[license-badge]: https://img.shields.io/crates/l/async-zmq
[build-badge]: https://img.shields.io/github/actions/workflow/status/rdelfin/async-zmq/main.yml?branch=main

**`async-zmq`** is a high-level, runtime-agnostic asynchronous interface for the \[`zmq`] crate.
It seamlessly integrates with any async runtime (**tokio**, **async-std**, etc.), making ZeroMQ simple, safe, and ergonomic in async Rust.

---

## 🚀 Features

* Support for all ZeroMQ socket types.
* Stream and Sink interfaces for receiving and sending multipart messages.
* Compatible with any async runtime.
* CURVE encryption and ZAP authentication support.

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

```rust
use async_zmq::Context;

let ctx = Context::new();
let xpub = async_zmq::xpublish("inproc://example")?.with_context(&ctx).bind()?;
let sub = async_zmq::subscribe("inproc://example")?.with_context(&ctx).connect()?;
```

---

## 🔐 CURVE Authentication

To enable secure communication between nodes, `async-zmq` supports CURVE encryption and authentication.

### 🛠 Requirements

* You **must** have `libsodium` and a ZeroMQ build that supports CURVE.
* Install `libzmq3-dev` and `libsodium-dev` (see [Installation](#-installation)).

### Generating Key Pair

```rust
use async_zmq::CurveKeyPair;

let keys = CurveKeyPair::new()?;
println!("Public: {:?}", keys.public_key);
println!("Secret: {:?}", keys.secret_key);
```

### Configuring a CURVE-Enabled Server (Publisher)

```rust
let mut socket = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;

socket.set_curve_server(true)?;
socket.set_curve_secretkey(&keys.secret_key)?;
socket.set_curve_publickey(&keys.public_key)?;
socket.set_zap_domain("global")?;
```

---

## 🔐 ZAP Authentication Handler

ZeroMQ’s ZAP protocol allows fine-grained authentication. A ZAP handler is required for CURVE to work.

### ZAP Handler Example

```rust
fn zap_auth_handler() -> async_zmq::Result<()> {
    let ctx = async_zmq::Context::new();
    let zap = ctx.socket(zmq::REP)?;
    zap.bind("inproc://zeromq.zap.01")?;

    loop {
        let request = zap.recv_multipart(0)?;
        if request.len() >= 7 && &request[5] == b"CURVE" {
            let client_key = String::from_utf8_lossy(&request[6]);
            println!("Client key: {}", client_key);

            let response = vec![
                "1.0", &request[1], "200", "OK", "user", "authenticated"
            ];
            zap.send_multipart(&response, 0)?;
        } else {
            let response = vec![
                "1.0", &request[1], "400", "Invalid", "", ""
            ];
            zap.send_multipart(&response, 0)?;
        }
    }
}
```

> Run the ZAP handler in a background thread before starting CURVE-enabled sockets.

---

## 🔒 Full Secure Publisher Example

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

## 📚 Documentation

* [ZeroMQ Guide](https://zguide.zeromq.org)
* [zmq crate](https://crates.io/crates/zmq)
* [async-std](https://crates.io/crates/async-std)
* [tokio](https://crates.io/crates/tokio)

---

## 🪪 License

Licensed under [MIT](LICENSE).

---
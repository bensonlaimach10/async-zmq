//! REQ socket module of Request-reply pattern in ZMQ
//!
//! Use the [`request`] function to instantiate a request socket.
//!
//! A request socket must be paired with a [`reply`] or [`router`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::Result;
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::request("tcp://127.0.0.1:5555")?.connect()?;
//!
//!     zmq.send(vec!["broadcast message"]).await?;
//!     let msg = zmq.recv().await?;
//!     Ok(())
//! }
//! ```
//!
//! [`reply`]: ../reply/index.html
//! [`router`]: ../router/index.html
//! [`request`]: fn.request.html

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Multipart, MultipartIter, Sender, SocketBuilder},
    RequestReplyError, SocketError,
};
use futures::future::poll_fn;
use std::sync::atomic::{AtomicBool, Ordering};
use zmq::{Message, SocketType};

/// Create a ZMQ socket with REQ type
pub fn request<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Request<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::REQ, endpoint))
}

/// The async wrapper of ZMQ socket with REQ type
pub struct Request<I: Iterator<Item = T> + Unpin, T: Into<Message>> {
    inner: Sender<I, T>,
    received: AtomicBool,
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Request<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self {
            inner: Sender {
                socket: ZmqSocket::from(socket),
                buffer: None,
            },
            received: AtomicBool::new(false),
        }
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Request<I, T> {
    /// Send request to REP/ROUTER socket. This should be the first method to be called, and then
    /// continue with send/receive pattern in synchronous way.
    pub async fn send<S: Into<MultipartIter<I, T>>>(
        &self,
        msg: S,
    ) -> Result<(), RequestReplyError> {
        let mut msg = msg.into();
        poll_fn(move |cx| self.inner.socket.send(cx, &mut msg)).await?;
        self.received.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Receive reply from REP/ROUTER socket. [`send`](#method.send) must be called first in order to receive reply.
    pub async fn recv(&self) -> Result<Multipart, RequestReplyError> {
        let msg = poll_fn(|cx| self.inner.socket.recv(cx)).await?;
        self.received.store(true, Ordering::Relaxed);
        Ok(msg)
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        self.inner.socket.as_socket()
    }

    /// Set the CURVE server flag on the socket.
    pub fn set_curve_server(&mut self, enabled: bool) -> Result<&mut Self, zmq::Error> {
        self.inner.socket.as_socket().set_curve_server(enabled)?;
        Ok(self)
    }

    /// Set the CURVE public key on the socket.
    pub fn set_curve_publickey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.inner.socket.as_socket().set_curve_publickey(key)?;
        Ok(self)
    }

    /// Set the CURVE secret key on the socket.
    pub fn set_curve_secretkey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.inner.socket.as_socket().set_curve_secretkey(key)?;
        Ok(self)
    }

    /// Set the CURVE server key on the socket.
    pub fn set_curve_serverkey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.inner.socket.as_socket().set_curve_serverkey(key)?;
        Ok(self)
    }

    /// Set the ZAP domain for authentication.
    pub fn set_zap_domain(&mut self, domain: &str) -> Result<&mut Self, zmq::Error> {
        self.inner.socket.as_socket().set_zap_domain(domain)?;
        Ok(self)
    }

    /// Set the send high water mark for the socket.
    /// The high water mark is a hard limit on the maximum number of outstanding messages
    /// ØMQ shall queue in memory for any single peer that the specified socket is communicating with.
    pub fn set_send_hwm(&mut self, value: i32) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_sndhwm(value)?;
        Ok(self)
    }

    /// Get the send high water mark for the socket.
    pub fn get_send_hwm(&self) -> Result<i32, zmq::Error> {
        self.as_raw_socket().get_sndhwm()
    }

    /// Set the receive high water mark for the socket.
    /// The high water mark is a hard limit on the maximum number of outstanding messages
    /// ØMQ shall queue in memory for any single peer that the specified socket is communicating with.
    pub fn set_receive_hwm(&mut self, value: i32) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_rcvhwm(value)?;
        Ok(self)
    }

    /// Get the receive high water mark for the socket.
    pub fn get_receive_hwm(&self) -> Result<i32, zmq::Error> {
        self.as_raw_socket().get_rcvhwm()
    }
}

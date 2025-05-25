//! PUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`publish`] function to instantiate a publish socket and use methods
//! from the [`Sink`]/[`SinkExt`] traits.
//!
//! A publish socket must be paired with a [`subscribe`] or [`xsubscribe`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, SinkExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     zmq.send(vec!["topic", "broadcast message"].into()).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`subscribe`]: ../subscribe/index.html
//! [`xsubscribe`]: ../xsubscribe/index.html
//! [`publish`]: fn.publish.html
//! [`Sink`]: ../trait.Sink.html
//! [`SinkExt`]: ../trait.SinkExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Message, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{MultipartIter, Sender, SocketBuilder},
    SendError, Sink, SocketError,
};

/// Create a ZMQ socket with PUB type
pub fn publish<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Publish<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::PUB, endpoint))
}

/// The async wrapper of ZMQ socket with PUB type
pub struct Publish<I: Iterator<Item = T> + Unpin, T: Into<Message>>(Sender<I, T>);

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Publish<I, T> {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        self.0.socket.as_socket()
    }

    /// Set the CURVE server flag on the socket.
    pub fn set_curve_server(&mut self, enabled: bool) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_curve_server(enabled)?;
        Ok(self)
    }

    /// Set the CURVE public key on the socket.
    pub fn set_curve_publickey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_curve_publickey(key)?;
        Ok(self)
    }

    /// Set the CURVE secret key on the socket.
    pub fn set_curve_secretkey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_curve_secretkey(key)?;
        Ok(self)
    }

    /// Set the CURVE server key on the socket.
    pub fn set_curve_serverkey(&mut self, key: &[u8]) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_curve_serverkey(key)?;
        Ok(self)
    }

    /// Set the ZAP domain for authentication.
    pub fn set_zap_domain(&mut self, domain: &str) -> Result<&mut Self, zmq::Error> {
        self.as_raw_socket().set_zap_domain(domain)?;
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
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Publish<I, T> {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }

    fn start_send(self: Pin<&mut Self>, item: MultipartIter<I, T>) -> Result<(), Self::Error> {
        Pin::new(&mut self.get_mut().0)
            .start_send(item)
            .map_err(Into::into)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_close(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Publish<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self(Sender {
            socket: ZmqSocket::from(socket),
            buffer: None,
        })
    }
}

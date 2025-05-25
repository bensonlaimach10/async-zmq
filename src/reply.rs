//! REP socket module of Request-reply pattern in ZMQ
//!
//! Use the [`reply`] function to instantiate a reply socket and use methods from
//! the [`Stream`]/[`StreamExt`] traits.
//!
//! A reply socket must be paired with a [`request`] or [`dealer`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::Result;
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::reply("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     let msg = zmq.recv().await?;
//!     zmq.send(vec!["broadcast message"]).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`dealer`]: ../dealer/index.html
//! [`request`]: ../request/index.html
//! [`reply`]: fn.reply.html

use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use zmq::{Message, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Multipart, MultipartIter, Sender, SocketBuilder},
    RecvError, RequestReplyError, SocketError,
};

use futures::{future::poll_fn, Stream};

/// Create a ZMQ socket with REP type
pub fn reply<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Reply<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::REP, endpoint))
}

/// The async wrapper of ZMQ socket with REP type
pub struct Reply<I: Iterator<Item = T> + Unpin, T: Into<Message>> {
    inner: Sender<I, T>,
    received: AtomicBool,
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Reply<I, T> {
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

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Reply<I, T> {
    /// Receive request from REQ/DEALER socket. This should be the first method to be called, and then
    /// continue with receive/send pattern in synchronous way.
    pub async fn recv(&self) -> Result<Multipart, RequestReplyError> {
        let msg = poll_fn(|cx| self.inner.socket.recv(cx)).await?;
        self.received.store(true, Ordering::Relaxed);
        Ok(msg)
    }

    /// Send reply to REQ/DEALER socket. [`recv`](#method.recv) must be called first in order to reply.
    pub async fn send<S: Into<MultipartIter<I, T>>>(
        &self,
        msg: S,
    ) -> Result<(), RequestReplyError> {
        let mut msg = msg.into();
        poll_fn(move |cx| self.inner.socket.send(cx, &mut msg)).await?;
        self.received.store(false, Ordering::Relaxed);
        Ok(())
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

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Stream for Reply<I, T> {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(futures::ready!(self.inner.socket.recv(cx))?)))
    }
}

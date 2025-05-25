//! CURVE security module for ZMQ
//!
//! This module provides types and functions for working with CURVE security in ZMQ.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, CurveKeyPair};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     // Generate a new CURVE key pair
//!     let key_pair = CurveKeyPair::new()?;
//!     
//!     // Access the public and secret keys
//!     println!("Public key: {:?}", key_pair.public_key);
//!     
//!     Ok(())
//! }
//! ```

use std::fmt;
use std::ops::{Deref, DerefMut};

/// A wrapper around zmq::CurveKeyPair that provides a more convenient API.
///
/// This struct holds a CURVE key pair for use with ZMQ CURVE security.
/// Each key pair consists of a public key and a secret key.
pub struct CurveKeyPair(zmq::CurveKeyPair);

impl CurveKeyPair {
    /// Create a new CURVE key pair.
    ///
    /// This generates a new public/secret key pair for use with CURVE security.
    pub fn new() -> Result<Self, zmq::Error> {
        Ok(Self(zmq::CurveKeyPair::new()?))
    }
}

impl Deref for CurveKeyPair {
    type Target = zmq::CurveKeyPair;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurveKeyPair {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for CurveKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CurveKeyPair")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

impl From<zmq::CurveKeyPair> for CurveKeyPair {
    fn from(ckp: zmq::CurveKeyPair) -> Self {
        Self(ckp)
    }
}

impl AsRef<zmq::CurveKeyPair> for CurveKeyPair {
    fn as_ref(&self) -> &zmq::CurveKeyPair {
        &self.0
    }
} 
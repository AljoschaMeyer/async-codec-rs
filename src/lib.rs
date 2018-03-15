//! Traits for composable encoding and decoding via AsyncRead/AsyncWrites.
#![deny(missing_docs)]

extern crate futures_core;
extern crate futures_io;

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use futures_core::Poll;
use futures_core::task::Context;
use futures_io::{AsyncWrite, AsyncRead, Error as FutIoErr};

/// A trait for types that asynchronously encode into an `AsyncWrite`.
pub trait AsyncEncode<W: AsyncWrite> {
    /// Call `writer.poll_write` exactly once with encoded data, propagating any `Err` and
    /// `Pending`, and returning how many bytes were written.
    ///
    /// After the value has been fully encoded, the next call to this must return `Ok(Ready(0))`.
    /// If `writer.poll_write` returns `Ok(Ready(0))` even though the value has not been fully
    /// encoded, this must return an error of kind `WriteZero`.
    fn poll_encode(&mut self, cx: &mut Context, writer: &mut W) -> Poll<usize, FutIoErr>;
}

/// An `AsyncEncode` that can precompute how many bytes of encoded data it produces.
pub trait AsyncEncodeLen<W: AsyncWrite>: AsyncEncode<W> {
    /// Return the exact number of bytes this will still write.
    fn remaining_bytes(&self) -> usize;
}

/// A trait for types can be asynchronously decoded from an `AsyncRead`.
pub trait AsyncDecode<R: AsyncRead> {
    /// The type of the value to decode.
    type Item;
    /// An error indicating how decoding can fail.
    type Error;

    /// Call `reader.poll_read` exactly once, propgating any `Err` and `Pending`, and return how
    /// many bytes have been read, as well as the decoded value, once decoding is done.
    ///
    /// This method may not be called after a value has been decoded.
    ///
    /// If `reader.poll_read` returns `Ok(Ready(0))` even though the value has not been fully
    /// decoded, this must return an error of kind `UnexpectedEof`.
    fn poll_decode(&mut self,
                   cx: &mut Context,
                   reader: &mut R)
                   -> Poll<(Option<Self::Item>, usize), DecodeError<Self::Error>>;
}

/// An error that occured during decoding.
#[derive(Debug)]
pub enum DecodeError<E> {
    /// An error propagated from the underlying reader.
    ReaderError(FutIoErr),
    /// An error describing why the read data could not be decoded into a value.
    DataError(E),
}

impl<E: Display> Display for DecodeError<E> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            DecodeError::ReaderError(ref err) => write!(f, "Decode reader error: {}", err),
            DecodeError::DataError(ref err) => write!(f, "Decode data error: {}", err),
        }
    }
}

impl<E: Error> Error for DecodeError<E> {
    fn description(&self) -> &str {
        match *self {
            DecodeError::ReaderError(ref err) => err.description(),
            DecodeError::DataError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            DecodeError::ReaderError(ref err) => Some(err),
            DecodeError::DataError(ref err) => Some(err),
        }
    }
}

impl<E> From<FutIoErr> for DecodeError<E> {
    fn from(err: FutIoErr) -> DecodeError<E> {
        DecodeError::ReaderError(err)
    }
}

//! Traits for composable encoding and decoding via AsyncRead/AsyncWrites.
#![deny(missing_docs)]

extern crate futures_core;
extern crate futures_io;

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use futures_core::task::Context;
use futures_io::{AsyncWrite, AsyncRead, Error as FutIoErr};

/// The return value for `poll_encode`.
pub enum PollEnc<S> {
    /// The encoder has been run to completion, the last call to `poll_encode` wrote this many bytes.
    Done(usize),
    /// Encoding is not done yet, but a non-zero number of bytes was written.
    Progress(S, usize),
    /// Encoding can not make progress, because the writer would block.
    /// The current task is scheduled to be awoken when progress can be made.
    Pending(S),
    /// The writer emitted an error.
    Errored(FutIoErr),
}

/// A trait for types that asynchronously encode into an `AsyncWrite`.
pub trait AsyncEncode
    where Self: Sized
{
    /// Call `writer.poll_write` once with encoded data, propagating any `Err` and
    /// `Pending`, and returning how many bytes were written.
    ///
    /// This consumes ownership of the encoder. If encoding did not terminate, the return value
    /// contains a new encoder that will resume at the correct point.
    ///
    /// If `writer.poll_write` returns `Ok(Ready(0))` even though the value has not been fully
    /// encoded, this must return an error of kind `WriteZero`.
    fn poll_encode<W: AsyncWrite>(self, cx: &mut Context, writer: &mut W) -> PollEnc<Self>;
}

/// An `AsyncEncode` that can precompute how many bytes of encoded data it produces.
pub trait AsyncEncodeLen: AsyncEncode {
    /// Return the exact number of bytes this will still write.
    fn remaining_bytes(&self) -> usize;
}

/// The return value for `poll_decode`.
pub enum PollDec<T, S, E> {
    /// The decoder has run to completion, yielding an item of type `T`. The second value is the
    /// number of bytes that were read in the last call to `poll_read`.
    Done(T, usize),
    /// Decoding is not done yet, but a non-zero number of bytes was read.
    Progress(S, usize),
    /// Decoding can not make progress, because the reader would block.
    /// /// The current task is scheduled to be awoken when progress can be made.
    Pending(S),
    /// An error occured during encoding.
    Errored(DecodeError<E>),
}

/// A trait for types can be asynchronously decoded from an `AsyncRead`.
pub trait AsyncDecode
    where Self: Sized
{
    /// The type of the value to decode.
    type Item;
    /// An error indicating how decoding can fail.
    type Error;

    /// Call `reader.poll_read` exactly once, propgating any `Err` and `Pending`, and return how
    /// many bytes have been read, as well as the decoded value, once decoding is done.
    ///
    /// This consumes ownership of the decoder. If decoding did not terminate, the return value
    /// contains a new decoder that will resume at the correct point.
    ///
    /// If `reader.poll_read` returns `Ok(Ready(0))` even though the value has not been fully
    /// decoded, this must return an error of kind `UnexpectedEof`.
    fn poll_decode<R: AsyncRead>(self,
                                 cx: &mut Context,
                                 reader: &mut R)
                                 -> PollDec<Self::Item, Self, Self::Error>;
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

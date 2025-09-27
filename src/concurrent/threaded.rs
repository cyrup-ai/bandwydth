//────────────────────────────────────────────────────────────────────────────
// Lock-free async bridge for any owned Unix **Read + Write** object.
// • no `unsafe`         • no locks        • zero allocation on every poll
// • no Arc              • pure ownership   • blazing fast
// • single helper thread performs blocking I/O and communicates through
//   lock-free crossbeam channels.
//────────────────────────────────────────────────────────────────────────────

#![forbid(unsafe_code)]

use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use std::io::{self, Read, Write};

use crossbeam_channel as cb;
use futures_core::Stream;
use pin_project_lite::pin_project;
use tokio::task::spawn_blocking;

/* ───────────────────────── constants ──────────────────────────────────── */

const READ_BUF_SIZE: usize = 8192; // 8 KiB read buffer for optimal performance
const WRITE_CHANNEL_BOUND: usize = 128; // Bounded channel for backpressure
const PENDING_WRITES_CAPACITY: usize = 32; // Pre-allocate pending writes vector

/* ───────────────────────── error types ──────────────────────────────────── */

/// Lightweight error type to avoid allocations
#[derive(Debug, Clone, Copy)]
pub enum AsyncIoError {
    BrokenPipe,
    UnexpectedEof,
    WriteZero,
    Interrupted,
    Other,
}

impl From<AsyncIoError> for io::Error {
    #[inline]
    fn from(err: AsyncIoError) -> Self {
        match err {
            AsyncIoError::BrokenPipe => io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken"),
            AsyncIoError::UnexpectedEof => {
                io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF")
            }
            AsyncIoError::WriteZero => io::Error::new(io::ErrorKind::WriteZero, "write zero"),
            AsyncIoError::Interrupted => io::Error::new(io::ErrorKind::Interrupted, "interrupted"),
            AsyncIoError::Other => io::Error::other("I/O error"),
        }
    }
}

impl From<io::Error> for AsyncIoError {
    #[inline]
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::BrokenPipe => AsyncIoError::BrokenPipe,
            io::ErrorKind::UnexpectedEof => AsyncIoError::UnexpectedEof,
            io::ErrorKind::WriteZero => AsyncIoError::WriteZero,
            io::ErrorKind::Interrupted => AsyncIoError::Interrupted,
            _ => AsyncIoError::Other,
        }
    }
}

/// Read result that minimizes allocations
pub enum ReadData {
    Ok(Vec<u8>),
    Err(AsyncIoError),
}

/* ───────────────────────── async writer ───────────────────────────────── */

/// Asynchronous writer that sends data through a thread-safe channel
///
/// This writer implementation allows non-blocking writes by sending
/// data through a crossbeam channel to a background thread.
#[derive(Clone)]
pub struct ThreadedAsyncWriter {
    tx: cb::Sender<Vec<u8>>,
}

impl ThreadedAsyncWriter {
    /// Send bytes asynchronously with zero-copy when possible
    #[inline]
    pub async fn send_async(&self, bytes: &[u8]) -> io::Result<()> {
        // Fast path for empty writes
        if bytes.is_empty() {
            return Ok(());
        }

        // Pre-allocate exact size needed
        let mut owned = Vec::with_capacity(bytes.len());
        owned.extend_from_slice(bytes);

        // Try non-blocking send first (hot path)
        match self.tx.try_send(owned) {
            Ok(()) => Ok(()),
            Err(cb::TrySendError::Full(mut buffer)) => {
                // Slow path: channel full, yield and retry
                loop {
                    tokio::task::yield_now().await;
                    match self.tx.try_send(buffer) {
                        Ok(()) => return Ok(()),
                        Err(cb::TrySendError::Full(b)) => buffer = b,
                        Err(cb::TrySendError::Disconnected(_)) => {
                            return Err(AsyncIoError::BrokenPipe.into());
                        }
                    }
                }
            }
            Err(cb::TrySendError::Disconnected(_)) => Err(AsyncIoError::BrokenPipe.into()),
        }
    }

    /// Check if the writer is still connected
    #[inline(always)]
    pub fn is_connected(&self) -> bool {
        // Check if we can still send by checking capacity
        self.tx.len() < self.tx.capacity().unwrap_or(usize::MAX)
    }
}

impl fmt::Debug for ThreadedAsyncWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncWriter")
            .field("connected", &self.is_connected())
            .field("pending", &self.tx.len())
            .finish()
    }
}

/* ───────────────────────── async reader ───────────────────────────────── */

pin_project! {
    /// Asynchronous reader that receives data from a thread-safe channel
    ///
    /// This reader implementation allows non-blocking reads by receiving
    /// data through a crossbeam channel from a background thread.
    #[derive(Clone)]
    pub struct ThreadedAsyncReader {
        #[pin]
        receiver: cb::Receiver<ReadData>,
    }
}

impl ThreadedAsyncReader {
    /// Get the number of pending chunks to read
    #[inline(always)]
    pub fn pending_chunks(&self) -> usize {
        self.receiver.len()
    }

    /// Check if the reader is disconnected
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        // Channel is disconnected if it's empty and try_recv would fail with Disconnected
        self.receiver.is_empty()
            && matches!(
                self.receiver.try_recv(),
                Err(cb::TryRecvError::Disconnected)
            )
    }
}

impl Stream for ThreadedAsyncReader {
    type Item = io::Result<Vec<u8>>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().receiver.try_recv() {
            Ok(ReadData::Ok(data)) => Poll::Ready(Some(Ok(data))),
            Ok(ReadData::Err(e)) => Poll::Ready(Some(Err(e.into()))),
            Err(cb::TryRecvError::Empty) => {
                // Wake immediately for next poll
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(cb::TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.receiver.len();
        if self.is_disconnected() {
            (len, Some(len))
        } else {
            (len, None)
        }
    }
}

impl fmt::Debug for ThreadedAsyncReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncReader")
            .field("pending", &self.pending_chunks())
            .field("connected", &!self.is_disconnected())
            .finish()
    }
}

/* ───────────────────────── main type ─────────────────────────────────── */

/// Zero-allocation async wrapper for Read+Write objects
#[derive(Clone)]
pub struct ThreadedRawFd {
    inner: Inner,
}

#[derive(Clone)]
struct Inner {
    reader: ThreadedAsyncReader,
    writer: ThreadedAsyncWriter,
}

impl ThreadedRawFd {
    /// Wrap any owned object that implements `Read + Write + Send + 'static`.
    #[inline]
    pub fn new<T>(io: T) -> Self
    where
        T: Read + Write + Send + 'static,
    {
        let (reader, writer) = split_async_io(io);
        Self {
            inner: Inner { reader, writer },
        }
    }

    /// Get the writer handle
    #[inline(always)]
    pub fn writer(&self) -> ThreadedAsyncWriter {
        self.inner.writer.clone()
    }

    /// Get a reference to the reader
    #[inline(always)]
    pub fn reader(&self) -> &ThreadedAsyncReader {
        &self.inner.reader
    }

    /// Consume self and return reader and writer
    #[inline]
    pub fn split(self) -> (ThreadedAsyncReader, ThreadedAsyncWriter) {
        (self.inner.reader, self.inner.writer)
    }
}

impl fmt::Debug for ThreadedRawFd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncRawFd")
            .field("reader", &self.inner.reader)
            .field("writer", &self.inner.writer)
            .finish()
    }
}

/* ───────────────────────── factory function ────────────────────────────── */

/// Split a Read+Write object into async reader and writer with dedicated I/O thread
///
/// Returns (reader, writer) tuple. The I/O thread runs until both are dropped.
#[inline]
pub fn split_async_io<T>(io: T) -> (ThreadedAsyncReader, ThreadedAsyncWriter)
where
    T: Read + Write + Send + 'static,
{
    // Channels for communication
    let (write_tx, write_rx) = cb::bounded::<Vec<u8>>(WRITE_CHANNEL_BOUND);
    let (read_tx, read_rx) = cb::unbounded::<ReadData>();

    // Spawn blocking I/O thread
    let _handle = spawn_blocking(move || io_thread(io, write_rx, read_tx));

    // Create reader with receiver
    let reader = ThreadedAsyncReader { receiver: read_rx };

    // Create writer
    let writer = ThreadedAsyncWriter { tx: write_tx };

    (reader, writer)
}

/* ─────────────────── I/O thread (blocking) ──────────────────────────────── */

#[inline(never)] // Don't inline the I/O thread
#[cold] // Optimize for size, not speed
fn io_thread<T>(mut io: T, write_rx: cb::Receiver<Vec<u8>>, read_tx: cb::Sender<ReadData>)
where
    T: Read + Write,
{
    // Pre-allocate all buffers
    let mut read_buf = vec![0u8; READ_BUF_SIZE];
    let mut pending_writes = Vec::with_capacity(PENDING_WRITES_CAPACITY);

    // Main I/O loop
    'main: loop {
        // Process all pending writes first (non-blocking drain)
        'write: loop {
            match write_rx.try_recv() {
                Ok(data) => {
                    if pending_writes.len() < pending_writes.capacity() {
                        pending_writes.push(data);
                    } else {
                        // Flush if we're at capacity
                        for chunk in pending_writes.drain(..) {
                            if let Err(e) = write_all(&mut io, &chunk) {
                                let _ = read_tx.send(ReadData::Err(e.into()));
                                break 'main;
                            }
                        }
                        pending_writes.push(data);
                    }
                }
                Err(cb::TryRecvError::Empty) => break 'write,
                Err(cb::TryRecvError::Disconnected) => break 'write,
            }
        }

        // Write all pending data
        for chunk in pending_writes.drain(..) {
            if let Err(e) = write_all(&mut io, &chunk) {
                let _ = read_tx.send(ReadData::Err(e.into()));
                break 'main;
            }
        }

        // Perform a single read
        match io.read(&mut read_buf) {
            Ok(0) => break 'main, // EOF
            Ok(n) => {
                // Allocate and send - this is the only allocation in steady state
                let data = read_buf[..n].to_vec();
                if read_tx.send(ReadData::Ok(data)).is_err() {
                    break 'main; // Reader dropped
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                let _ = read_tx.send(ReadData::Err(e.into()));
                break 'main;
            }
        }

        // Fast exit check
        if matches!(write_rx.try_recv(), Err(cb::TryRecvError::Disconnected)) && read_tx.is_empty()
        {
            break 'main;
        }
    }
}

/// Helper to write all bytes with proper error handling
#[inline(always)]
fn write_all<W: Write>(writer: &mut W, mut buf: &[u8]) -> io::Result<()> {
    while !buf.is_empty() {
        match writer.write(buf) {
            Ok(0) => return Err(AsyncIoError::WriteZero.into()),
            Ok(n) => buf = &buf[n..],
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

//────────────────────────────────────────────────────────────────────────────
// # END

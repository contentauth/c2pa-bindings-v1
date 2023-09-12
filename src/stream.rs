// Copyright 2023 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

use std::io::{Read, Seek, SeekFrom, Write};

use thiserror::Error;

use crate::error::C2paError;

pub type StreamResult<T> = std::result::Result<T, StreamError>;

#[repr(C)]
pub enum SeekMode {
    Start,
    End,
    Current,
}

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Io: {reason}")]
    Io { reason: String },
    #[error("Other: {reason}")]
    Other { reason: String },
    #[error("InternalStreamError")]
    InternalStreamError,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for StreamError {
    fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::Other {
            reason: err.reason.clone(),
        }
    }
}

impl From<C2paError> for StreamError {
    fn from(e: C2paError) -> Self {
        Self::Other {
            reason: e.to_string(),
        }
    }
}

/// This allows for a callback stream over the Uniffi interface.
/// Implement these stream functions in the foreign language
/// and this will provide Rust Stream trait implementations
/// This is necessary since the Rust traits cannot be implemented directly
/// as uniffi callbacks
pub trait Stream: Send + Sync {
    /// Read a stream of bytes from the stream
    fn read_stream(&self, length: u64) -> StreamResult<Vec<u8>>;
    /// Seek to a position in the stream
    fn seek_stream(&self, pos: i64, mode: SeekMode) -> StreamResult<u64>;
    /// Write a stream of bytes to the stream
    fn write_stream(&self, data: Vec<u8>) -> StreamResult<u64>;
}

impl Read for dyn Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes = self
            .read_stream(buf.len() as u64)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let len = bytes.len();
        buf.iter_mut().zip(bytes.drain(..)).for_each(|(dest, src)| {
            *dest = src;
        });
        Ok(len)
    }
}

impl Seek for dyn Stream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let (pos, mode) = match pos {
            SeekFrom::Current(pos) => (pos, SeekMode::Current),
            SeekFrom::Start(pos) => (pos as i64, SeekMode::Start),
            SeekFrom::End(pos) => (pos, SeekMode::End),
        };
        self.seek_stream(pos, mode)
            .map_err(|_| std::io::Error::last_os_error())
    }
}

impl Write for dyn Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self
            .write_stream(buf.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(len as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct StreamAdapter<'a> {
    pub reader: &'a mut dyn Stream,
}

impl<'a> StreamAdapter<'a> {
    pub fn from_stream(reader: &'a mut dyn Stream) -> Self {
        Self { reader }
    }
}

impl<'a> c2pa::CAIRead for StreamAdapter<'a> {}

impl<'a> c2pa::CAIReadWrite for StreamAdapter<'a> {}

impl<'a> Read for StreamAdapter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes = self
            .reader
            .read_stream(buf.len() as u64)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let len = bytes.len();
        buf.iter_mut().zip(bytes.drain(..)).for_each(|(dest, src)| {
            *dest = src;
        });
        //println!("read: {:?}", len);
        Ok(len)
    }
}

impl<'a> Seek for StreamAdapter<'a> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let (pos, mode) = match pos {
            SeekFrom::Current(pos) => (pos, SeekMode::Current),
            SeekFrom::Start(pos) => (pos as i64, SeekMode::Start),
            SeekFrom::End(pos) => (pos, SeekMode::End),
        };
        //println!("Stream Seek {}", pos);
        self.reader
            .seek_stream(pos, mode)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl<'a> Write for StreamAdapter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self
            .reader
            .write_stream(buf.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(len as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_stream::TestStream;

    #[test]
    fn test_stream_read() {
        let mut test = TestStream::from_memory(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut stream = StreamAdapter::from_stream(&mut test);
        let mut buf = [0u8; 5];
        let len = stream.read(&mut buf).unwrap();
        assert_eq!(len, 5);
        assert_eq!(buf, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_stream_seek() {
        let mut test = TestStream::from_memory(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut stream = StreamAdapter { reader: &mut test };
        let pos = stream.seek(SeekFrom::Start(5)).unwrap();
        assert_eq!(pos, 5);
        let mut buf = [0u8; 5];
        let len = stream.read(&mut buf).unwrap();
        assert_eq!(len, 5);
        assert_eq!(buf, [5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_stream_write() {
        let mut test = TestStream::new();
        let mut stream = StreamAdapter { reader: &mut test };
        let len = stream.write(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(len, 5);
        stream.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0u8; 5];
        let len = stream.read(&mut buf).unwrap();
        assert_eq!(len, 5);
        assert_eq!(buf, [0, 1, 2, 3, 4]);
    }
}

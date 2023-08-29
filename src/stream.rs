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

use std::io::{Read, Seek, SeekFrom};
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
    #[error("IoError")]
    IoError,
    #[error("Other Error: {reason}")]
    Other { reason: String },
    #[error("InternalStreamError")]
    InternalStreamError,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for StreamError {
    fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
        let err = Self::Other {
            reason: err.reason.clone(),
        };
        println!("{:#}", err);
        err
    }
}

impl From<C2paError> for StreamError {
    fn from(e: C2paError) -> Self {
        Self::Other {
            reason: e.to_string(),
        }
    }
}

pub trait ReadStream: Send + Sync {
    fn read_stream(&self, length: u64) -> StreamResult<Vec<u8>>;
    fn seek_stream(&self, pos: i64, mode: SeekMode) -> StreamResult<u64>;
}

impl Read for dyn ReadStream {
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

impl Seek for dyn ReadStream {
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

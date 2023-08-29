// Copyright 2022 Adobe. All rights reserved.
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

use std::{
    ffi::{c_int, c_long, CStr, CString},
    io::{Read, Seek, SeekFrom},
    os::raw::c_char,
};

use crate::{ManifestStoreReader, ReadStream, SeekMode, StreamError, StreamResult};

type ReadCallback =
    unsafe extern "C" fn(context: *const StreamContext, data: *mut u8, len: usize) -> isize;

type SeekCallback =
    unsafe extern "C" fn(context: *const StreamContext, offset: c_long, mode: c_int) -> c_int;

#[repr(C)]
pub struct StreamContext {
    _priv: (),
}

#[repr(C)]
pub struct C2paConfigC {
    pub data_dir: *const c_char, // optional UTF-8 path
    pub dest_option: u8,
    pub ingredient_option: u8,
}

impl C2paConfigC {
    fn _new() -> Self {
        Self {
            data_dir: std::ptr::null(),
            dest_option: 0,
            ingredient_option: 0,
        }
    }
    // unsafe fn to_rust(&self) -> C2paConfig {
    //     crate::C2paConfig {
    //         data_dir: if self.data_dir.is_null() {
    //             None
    //         } else {
    //             Some(from_c_str(self.data_dir))
    //         },
    //         dest_option: self.dest_option.try_into().unwrap_or(DestOption::Embed),
    //         ingredient_option: self
    //             .ingredient_option
    //             .try_into()
    //             .unwrap_or(IngredientOption::None),
    //     }
    // }
}
#[repr(C)]
pub struct C2paReader {
    context: Box<StreamContext>,
    read: ReadCallback,
    seek: SeekCallback,
}

impl C2paReader {
    pub fn new(context: *mut StreamContext, read: ReadCallback, seek: SeekCallback) -> Self {
        Self {
            context: unsafe { Box::from_raw(context) },
            read,
            seek,
        }
    }
}

impl Read for C2paReader {
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

impl Seek for C2paReader {
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

impl crate::ReadStream for C2paReader {
    fn read_stream(&self, length: u64) -> StreamResult<Vec<u8>> {
        let mut buffer = vec![0u8; length as usize];
        let context = &(*self.context);
        let bytes_read = unsafe { (self.read)(context, buffer.as_mut_ptr(), length as usize) };
        if bytes_read < 0 {
            println!("read_stream error{}", bytes_read);
            return Err(StreamError::IoError);
        }
        unsafe { buffer.set_len(bytes_read as usize) }
        Ok(buffer)
    }

    fn seek_stream(&self, pos: i64, mode: SeekMode) -> StreamResult<u64> {
        let context: &StreamContext = &(*self.context);
        let new_pos = unsafe { (self.seek)(context, pos as c_long, mode as c_int) };
        if new_pos < 0 {
            return Err(StreamError::IoError);
        }
        Ok(new_pos as u64)
    }
}

#[no_mangle]
pub unsafe extern "C" fn c2pa_create_reader(
    context: *mut StreamContext,
    read: ReadCallback,
    seek: SeekCallback,
) -> *mut C2paReader {
    Box::into_raw(Box::new(C2paReader::new(context, read, seek)))
}

// Internal routine to convert a *const c_char to a rust String
unsafe fn _from_c_str(s: *const c_char) -> String {
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

// Internal routine to return a rust String reference to C as *mut c_char
// The returned value MUST be released by calling release_string
// and it is no longer valid after that call.
unsafe fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns a version string for logging
///
/// # Safety
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_version() -> *mut c_char {
    to_c_string(crate::version())
}

/// Returns a JSON array of supported file format extensions
///
/// # Safety
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_supported_extensions() -> *mut c_char {
    to_c_string(serde_json::to_string(&crate::supported_extensions()).unwrap_or_default())
}

/// Verify a stream and return a ManifestStore report
///
/// # Errors
/// Returns an error field if there were errors
///
/// # Safety
/// Reads from a null terminated C string
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_verify_stream(reader: C2paReader) -> *mut c_char {
    let manifest_store = ManifestStoreReader::new();
    let result = manifest_store.read("image/jpeg", reader);
    let str = match result {
        Ok(json) => json,
        Err(e) => format!("manifest store read error {:?}", e),
    };
    to_c_string(str)
}

/// Releases a string allocated by Rust
///
/// # Safety
/// Reads from null terminated C strings
/// The string must not have been modified in C
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    drop(CString::from_raw(s));
}

/// Releases a C2paReader
///
/// # Safety
/// Reads from null terminated C strings
/// The string must not have been modified in C
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_reader(reader: *mut C2paReader) {
    if reader.is_null() {
        return;
    }
    drop(Box::from_raw(reader));
}

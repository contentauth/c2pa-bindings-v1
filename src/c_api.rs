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

use std::{
    ffi::{c_char, c_int, c_long, CStr, CString},
    io::{Read, Seek, SeekFrom, Write},
};

use crate::{C2paError, ManifestStoreReader};

/// Defines a callback to read from a stream
type ReadCallback =
    unsafe extern "C" fn(context: *const StreamContext, data: *mut u8, len: usize) -> isize;

/// Defines a callback to seek to an offset in a stream
type SeekCallback =
    unsafe extern "C" fn(context: *const StreamContext, offset: c_long, mode: c_int) -> c_int;

/// Defines a callback to write to a stream
type WriteCallback =
    unsafe extern "C" fn(context: *const StreamContext, data: *const u8, len: usize) -> isize;

#[repr(C)]
#[derive(Debug)]
/// An Opaque struct to hold a context value for the stream callbacks
pub struct StreamContext {
    _priv: (),
}

#[repr(C)]
pub struct _C2paConfigC {
    pub data_dir: *const c_char, // optional UTF-8 path
    pub dest_option: u8,
    pub ingredient_option: u8,
}

impl _C2paConfigC {
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
/// A C2paStream is a Rust Read/Write/Seek stream that can be used in C
#[derive(Debug)]
pub struct C2paStream {
    context: Box<StreamContext>,
    read: ReadCallback,
    seek: SeekCallback,
    write: WriteCallback,
}

impl C2paStream {
    /// Creates a new C2paStream from context with callbacks
    /// # Arguments
    /// * `context` - a pointer to a StreamContext
    /// * `read` - a ReadCallback to read from the stream
    /// * `seek` - a SeekCallback to seek in the stream
    /// * `write` - a WriteCallback to write to the stream
    /// # Safety
    ///     The context must remain valid for the lifetime of the C2paStream
    ///     The read, seek, and write callbacks must be valid for the lifetime of the C2paStream
    ///     The resulting C2paStream must be released by calling c2pa_release_stream
    pub unsafe fn new(
        context: *mut StreamContext,
        read: ReadCallback,
        seek: SeekCallback,
        write: WriteCallback,
    ) -> Self {
        Self {
            context: unsafe { Box::from_raw(context) },
            read,
            seek,
            write,
        }
    }
}

impl Read for C2paStream {
    // implements Rust Read trait by calling back to the C read callback
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let context = &(*self.context);
        let bytes_read = unsafe { (self.read)(context, buf.as_mut_ptr(), buf.len()) };
        if bytes_read < 0 {
            println!("read_stream error{}", bytes_read);
            return Err(std::io::Error::last_os_error());
        }
        Ok(bytes_read as usize)
    }
}

impl Seek for C2paStream {
    // implements Rust Seek trait by calling back to the C seek callback
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let context: &StreamContext = &self.context;
        let (pos, whence) = match pos {
            SeekFrom::Current(pos) => (pos, 2),
            SeekFrom::Start(pos) => (pos as i64, 1),
            SeekFrom::End(pos) => (pos, 3),
        };
        let new_pos = unsafe { (self.seek)(context, pos as c_long, whence as c_int) };
        if new_pos < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(new_pos as u64)
    }
}

impl Write for C2paStream {
    // implements Rust Write trait by calling back to the C write callback
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let context = &(*self.context);
        let bytes_written = unsafe { (self.write)(context, buf.as_ptr(), buf.len()) };
        if bytes_written < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(bytes_written as usize)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(()) // todo: do we need to expose this?
    }
}

/// Creates a new C2paStream from context with callbacks
///
/// This allows implementing streams in other languages
///
/// # Arguments
/// * `context` - a pointer to a StreamContext
/// * `read` - a ReadCallback to read from the stream
/// * `seek` - a SeekCallback to seek in the stream
/// * `write` - a WriteCallback to write to the stream
///     
/// # Safety
/// The context must remain valid for the lifetime of the C2paStream
/// The resulting C2paStream must be released by calling c2pa_release_stream
///
#[no_mangle]
pub unsafe extern "C" fn c2pa_create_stream(
    context: *mut StreamContext,
    read: ReadCallback,
    seek: SeekCallback,
    write: WriteCallback,
) -> *mut C2paStream {
    Box::into_raw(Box::new(C2paStream::new(context, read, seek, write)))
}

// Internal routine to convert a *const c_char to a rust String
unsafe fn from_c_str(s: *const c_char) -> String {
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

// Internal routine to return a rust String reference to C as *mut c_char
// The returned value MUST be released by calling release_string
// and it is no longer valid after that call.
unsafe fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(e) => {
            C2paError::Ffi(e.to_string()).set_last();
            std::ptr::null_mut()
        }
    }
}

/// Returns the last error message
///
/// # Safety
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_error() -> *mut c_char {
    to_c_string(C2paError::last_message().unwrap_or_default())
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
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_verify_stream(reader: C2paStream) -> *mut c_char {
    let manifest_store = ManifestStoreReader::new();
    let result = manifest_store.read("image/jpeg", reader);
    let str = match result {
        Ok(json) => json,
        Err(e) => {
            e.set_last();
            return std::ptr::null_mut();
        }
    };
    to_c_string(str)
}

/// Create a new ManifestStoreReader
///
/// # Safety
/// The returned value MUST be released by calling release_manifest_reader
///
/// # Example
/// ```
/// use c2pa::ManifestStoreReader;
/// let reader = ManifestStoreReader::new();
/// ```
#[no_mangle]
pub unsafe extern "C" fn c2pa_manifest_reader_new() -> *mut ManifestStoreReader {
    let reader = ManifestStoreReader::new();
    Box::into_raw(Box::new(reader))
}

/// Read a manifest store from a stream
///
/// # Arguments
/// * `reader_ptr` - a pointer to a ManifestStoreReader
/// * `format` - the format of the manifest store
/// * `stream` - the stream to read from
///
/// # Returns
/// * `Result<String>` - the json representation of the manifest store
///
/// # Example
/// ```
/// use c2pa::ManifestStoreReader;
/// use std::io::Cursor;
///     
/// let reader = ManifestStoreReader::new();
/// let mut stream = Cursor::new("test".as_bytes());
/// let json = reader.read("image/jpeg", &mut stream);
/// ```
///
/// # Safety
/// Reads from null terminated C strings
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
///
#[no_mangle]
pub unsafe extern "C" fn c2pa_manifest_reader_read(
    reader_ptr: *mut *mut ManifestStoreReader,
    format: *const c_char,
    stream: C2paStream,
) -> *mut c_char {
    let reader = Box::from_raw(*reader_ptr);
    let format = from_c_str(format);
    let result = reader.read(&format, stream);
    let str = match result {
        Ok(json) => json,
        Err(e) => {
            e.set_last();
            return std::ptr::null_mut();
        }
    };
    *reader_ptr = Box::into_raw(reader);
    to_c_string(str)
}

/// Writes a resource from the manifest reader to a stream
///
/// # Arguments
/// * `reader_ptr` - a pointer to a ManifestStoreReader
/// * `manifest_label` - the manifest label
/// * `id` - the resource id
/// * `stream` - the stream to write to
///
/// # Example
/// ```
/// use c2pa::ManifestStoreReader;
/// use std::io::Cursor;
///
/// let reader = ManifestStoreReader::new();
/// let mut stream = Cursor::new("test".as_bytes());
/// reader.resource_write("manifest", "id", &mut stream);
/// ```
///
/// # Safety
/// Reads from null terminated C strings
///
/// # Errors
/// Returns an error field if there were errors
///
#[no_mangle]
pub unsafe extern "C" fn c2pa_manifest_reader_resource(
    reader_ptr: *mut *mut ManifestStoreReader,
    manifest_label: *const c_char,
    id: *const c_char,
    stream: C2paStream,
) {
    let reader = Box::from_raw(*reader_ptr);
    let manifest_label = from_c_str(manifest_label);
    let id = from_c_str(id);
    let result = reader.resource_write(&manifest_label, &id, stream);
    if let Err(e) = result {
        e.set_last();
    }
    *reader_ptr = Box::into_raw(reader);
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

/// Releases a C2paStream allocated by Rust
///
/// # Safety
/// Reads from null terminated C strings
/// The string must not have been modified in C
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_stream(stream: *mut C2paStream) {
    if stream.is_null() {
        return;
    }
    drop(Box::from_raw(stream));
}

/// Releases a C2paManifestReader allocated by Rust
///
/// # Safety
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_manifest_reader(reader: *mut C2paStream) {
    if reader.is_null() {
        return;
    }
    drop(Box::from_raw(reader));
}

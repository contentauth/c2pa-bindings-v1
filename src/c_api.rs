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

use crate::{C2paError, C2paSigner, ManifestStoreReader, ManifestBuilder, ManifestBuilderSettings, SignerConfig, SeekMode};

/// Defines a callback to read from a stream
type ReadCallback =
    unsafe extern "C" fn(context: *const StreamContext, data: *mut u8, len: usize) -> isize;

/// Defines a callback to seek to an offset in a stream
type SeekCallback =
    unsafe extern "C" fn(context: *const StreamContext, offset: c_long, mode: SeekMode) -> c_int;

/// Defines a callback to write to a stream
type WriteCallback =
    unsafe extern "C" fn(context: *const StreamContext, data: *const u8, len: usize) -> isize;

/// Defines a callback to sign data
type SignerCallback =
    unsafe extern "C" fn(data: *mut u8, len: usize, signature: *mut u8, sig_max_size: isize) -> isize;

#[repr(C)]
#[derive(Debug)]
/// An Opaque struct to hold a context value for the stream callbacks
pub struct StreamContext {
    _priv: (),
}

#[repr(C)]
pub struct ManifestBuilderSettingsC {
    pub claim_generator: *const c_char,
}

#[repr(C)]
/// Defines the configuration for a Signer
///
/// # Example
/// ```
/// use c2pa::SignerConfig;
/// let config = SignerConfig {
///    alg: "Rs256".to_string(),
///    certs: vec![vec![0; 10]],
///    time_authority_url: Some("http://example.com".to_string()),
///    use_ocsp: true,
/// };
pub struct SignerConfigC {
    /// Returns the algorithm of the Signer.
    pub alg: *const c_char,

    /// Returns the certificates as a Vec containing a Vec of DER bytes for each certificate.
    pub certs: *const c_char,

    /// URL for time authority to time stamp the signature
    pub time_authority_url: *const c_char,

    /// Try to fetch OCSP response for the signing cert if available
    pub use_ocsp: bool,
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
/// A C2paSignerCallback defines a signer in C to be called from Rust
#[derive(Debug)]
struct CSignerCallback {
    signer: SignerCallback
}

impl crate::SignerCallback for CSignerCallback {
    fn sign(&self, data: Vec<u8>) -> crate::StreamResult<Vec<u8>> {
        //println!("SignerCallback signing {:p} {}",self, data.len());
        let sig_max_size = 100000;
        let mut signature = vec![0; sig_max_size];
        //let mut signature: *mut u8 = std::ptr::null_mut();
        // This callback returns the size of the signature, if negative it means there was an error
        let sig: *mut u8 = signature.as_ptr() as *mut u8;
        let result = unsafe { (self.signer)(data.as_ptr() as *mut u8, data.len(), sig, sig_max_size as isize) };
        if result < 0 {
            // todo: return errors from callback
            return Err(crate::StreamError::Other{reason:"signer error".to_string()});
        }
        signature.truncate(result as usize);
        //println!("SignerCallback signing result {}", signature.len());
        //let signature = unsafe { Vec::from_raw_parts(signature, result as usize, result as usize) };
        Ok(signature)
    }
}

#[no_mangle]
pub unsafe extern "C" fn c2pa_create_signer(
    signer: SignerCallback,
    config: &SignerConfigC
) -> *mut C2paSigner {
    let config = SignerConfig {
        alg: from_c_str(config.alg).to_lowercase(),
        certs: from_c_str(config.certs).into_bytes(),
        time_authority_url: if config.time_authority_url.is_null() {
            None
        } else {
            Some(from_c_str(config.time_authority_url))
        },
        use_ocsp: config.use_ocsp,
    };
    let callback = Box::new(CSignerCallback { signer });
    let signer = C2paSigner::new(callback);
    match signer.configure(&config){
        Ok(_) => Box::into_raw(Box::new(signer)),
        Err(e) => {
            e.set_last();
            std::ptr::null_mut()
        }
    }
}


#[repr(C)]
/// A C2paStream is a Rust Read/Write/Seek stream that can be used in C
#[derive(Debug)]
pub struct C2paStream {
    context: Box<StreamContext>,
    read_callback: ReadCallback,
    seek_callback: SeekCallback,
    write_callback: WriteCallback,
}

impl C2paStream {
    /// Creates a new C2paStream from context with callbacks
    /// # Arguments
    /// * `context` - a pointer to a StreamContext
    /// * `read_callback` - a ReadCallback to read from the stream
    /// * `seek_callback` - a SeekCallback to seek in the stream
    /// * `write_callback` - a WriteCallback to write to the stream
    /// # Safety
    ///     The context must remain valid for the lifetime of the C2paStream
    ///     The read, seek, and write callbacks must be valid for the lifetime of the C2paStream
    ///     The resulting C2paStream must be released by calling c2pa_release_stream
    pub unsafe fn new(
        context: *mut StreamContext,
        read_callback: ReadCallback,
        seek_callback: SeekCallback,
        write_callback: WriteCallback,
    ) -> Self {
        Self {
            context: unsafe { Box::from_raw(context) },
            read_callback,
            seek_callback,
            write_callback,
        }
    }
}

impl Read for C2paStream {
    // implements Rust Read trait by calling back to the C read callback
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let context = &(*self.context);
        let bytes_read = unsafe { (self.read_callback)(context, buf.as_mut_ptr(), buf.len()) };
        if bytes_read < 0 {
            //println!("read_stream error{}", bytes_read);
            return Err(std::io::Error::last_os_error());
        }
        //println!("C2paStream.read {:p} context={:p} bytes read={}",self, context, bytes_read);
        Ok(bytes_read as usize)
    }
}

impl Seek for C2paStream {
    // implements Rust Seek trait by calling back to the C seek callback
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        //let context: &StreamContext = &self.context;
        let context = &(*self.context);
        let (pos, whence) = match pos {
            SeekFrom::Current(pos) => (pos, SeekMode::Current),
            SeekFrom::Start(pos) => (pos as i64, SeekMode::Start),
            SeekFrom::End(pos) => (pos, SeekMode::End),
        };
        //println!("C2paStream.seek {:p} context = {:p} whence = {:?} pos={}", self, context, whence, pos);
        let new_pos = unsafe { (self.seek_callback)(context, pos as c_long, whence) };
        if new_pos < 0 {
            return Err(std::io::Error::last_os_error());
        }
        //println!("C2paStream.seek {:p} context = {:p} whence = {:?} pos={}", self, context, whence, new_pos);
        Ok(new_pos as u64)
    }
}

impl Write for C2paStream {
    // implements Rust Write trait by calling back to the C write callback
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let context = &(*self.context);
        let bytes_written = unsafe { (self.write_callback)(context, buf.as_ptr(), buf.len()) };
        if bytes_written < 0 {
            return Err(std::io::Error::last_os_error());
        }
        //println!("C2paStream.write {}", bytes_written);
        Ok(bytes_written as usize)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(()) // todo: do we need to expose this?
    }
}

// unsafe impl Send for C2paStream {}

impl c2pa::CAIRead for C2paStream {}

impl c2pa::CAIReadWrite for C2paStream {}

// impl Stream for C2paStream {
//     fn read_stream(&self, len: u64) -> crate::StreamResult<Vec<u8>> {
//         self.read_callback(&*self.context, std::ptr::null_mut(), len as usize);
//         // let mut buf = vec![0; len as usize];
//         // let bytes_read = self.read(buf.as_mut_slice())
//         //     .map_err(|e| crate::StreamError::Other{reason: e.to_string()})?;
//         // buf.truncate(bytes_read);
//         // Ok(buf)
//     }

//     fn write_stream(&self, data: Vec<u8>) -> crate::StreamResult<u64> {
//         let count = self.write(data.as_slice())
//             .map_err(|e| crate::StreamError::Other{reason: e.to_string()})?;
//         Ok(count as u64)
//     }

//     fn seek_stream(&mut self, pos: i64, mode: c2pa::SeekMode) -> crate::StreamResult<u64> {
//         let whence = match mode {
//             c2pa::SeekMode::Start => SeekFrom::Start(pos as u64),
//             c2pa::SeekMode::Current => SeekFrom::Current(pos),
//             c2pa::SeekMode::End => SeekFrom::End(pos),
//         };
//         let new_pos = self.seek(whence)?;
//         Ok(new_pos)
//     }   
// }

// impl Into<Box<dyn Stream>> for C2paStream {
//     fn into(self) -> Box<dyn Stream> {
//         Box::new(StreamAdapter::from_stream(Box::new(self)))
//     }
// }


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
    stream: *mut C2paStream,
) {
    let reader = Box::from_raw(*reader_ptr);
    let stream = &mut *stream;
    let manifest_label = from_c_str(manifest_label);
    let id = from_c_str(id);
    let result = reader.resource_write(&manifest_label, &id, stream);
    if let Err(e) = result {
        e.set_last();
    }
    *reader_ptr = Box::into_raw(reader);
}


/// Create a ManifestBuilder
/// 
/// # Arguments
/// * `settings` - a pointer to a ManifestBuilderSettingsC
/// * `json` - a pointer to a null terminated JSON Manifest Definition
/// 
/// # Returns
/// * `Result<*mut ManifestBuilder>` - a pointer to a ManifestBuilder
///
/// # Safety
/// The returned value MUST be released by calling release_manifest_builder
///
/// # Example
/// ```
/// use c2pa::{ManifestBuilder, ManifestBuilderSettings};
/// let json = r#"{
///     "claim_generator": "test_generator",
///     "format": "image/jpeg",
///     "title": "test_title"
/// }"#;
/// let settings = ManifestBuilderSettings {
///    generator: "test".to_string(),
/// };
///     
///   let builder = ManifestBuilder::new(&settings);
///    builder.from_json(json);
/// ```
/// 

#[no_mangle]
pub unsafe extern "C" fn c2pa_create_manifest_builder(
        settings: &ManifestBuilderSettingsC,
        json: *const c_char,
) -> *mut ManifestBuilder {
    let json = from_c_str(json);
    let settings = ManifestBuilderSettings {
        generator: from_c_str(settings.claim_generator),
    };
    let builder = ManifestBuilder::new(&settings);
    builder.from_json(&json).unwrap();
    match builder.from_json(&json) {
        Ok(_) => Box::into_raw(Box::new(builder)),
        Err(e) => {
            e.set_last();
            return std::ptr::null_mut();
        }
    }
}

#[no_mangle]
/// Sign using a ManifestBuilder
/// 
/// # Arguments
/// * `builder` - a pointer to a ManifestBuilder
/// * `signer` - a pointer to a C2paSigner
/// * `input` - a pointer to a C2paStream
/// * `output` - optional pointer to a C2paStream
/// 
pub unsafe extern "C" fn c2pa_manifest_builder_sign(
    builder_ptr: *mut *mut ManifestBuilder,
    signer: *mut C2paSigner,
    input: *mut C2paStream,
    output: *mut C2paStream,
    ) -> c_int {
    let builder = Box::from_raw(*builder_ptr);
    let signer = Box::from_raw(signer);
    let mut input = Box::from_raw(input);
    let output = match output.is_null() {
        true => None,
        false => Some(&mut *output),
    };
    //println!("c2pa_manifest_builder_sign input {:p}, context {:p}", &(*input), input.context);
    input.seek(SeekFrom::Start(0)).unwrap();
    let result = builder.sign_cai_read(&(*signer), &mut (*input), output);  
    *builder_ptr = Box::into_raw(builder);
    Box::into_raw(signer);
    Box::into_raw(input);
    match result {
        Ok(_) => 0,
        Err(e) => {
            e.set_last();
            return -1;
        }
    }
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

/// Releases a ManifestStoreReader allocated by Rust
///
/// # Safety
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_manifest_reader(reader: *mut ManifestStoreReader) {
    if reader.is_null() {
        return;
    }
    drop(Box::from_raw(reader));
}

/// Releases a ManifestBuilder allocated by Rust
///
/// # Safety
/// can only be released once and is invalid after this call
#[no_mangle]
pub unsafe extern "C" fn c2pa_release_manifest_builder(builder: *mut ManifestBuilder) {
    if builder.is_null() {
        return;
    }
    drop(Box::from_raw(builder));
}

// pub unsafe extern "C" fn c2pa_release_box(object: *mut std::ffi::c_void) {
//     if object.is_null() {
//         return;
//     }
//     drop(Box::from_raw(object));
// }
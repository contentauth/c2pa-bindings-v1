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
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::{
    json_api::{add_manifest_to_file_json, ingredient_from_file_json, verify_from_file_json},
    response::Response,
    signer_info::SignerInfo,
};
use c2pa::Result;
use serde::Serialize;

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
        Err(_) => std::ptr::null_mut(),
    }
}

// convert a Result into JSON result string as *mut c_char
// The returned value MUST be released by calling release_string
// and it is no longer valid after that call.
unsafe fn result_to_c_string<T: Serialize>(result: Result<T>) -> *mut c_char {
    to_c_string(Response::from_result(result).to_string())
}

/// Returns a version string for logging
///
/// # Safety
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_version() -> *mut c_char {
    let version = format!(
        "{}/{} {}/{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        c2pa::NAME,
        c2pa::VERSION
    );
    to_c_string(version)
}

/// Returns a JSON array of supported file format extensions
///
/// # Safety
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_supported_formats() -> *mut c_char {
    let formats = "[\"jpeg\"]".to_string();
    to_c_string(formats)
}

/// Returns a true (1) if the file appears to have a valid manifest, else false (0)
///
/// # Safety
/// expects a valid null terminated path as input
#[no_mangle]
pub unsafe extern "C" fn c2pa_has_manifest(path: *const c_char) -> u8 {
    use c2pa::Ingredient;
    let info = Ingredient::from_file_info(from_c_str(path));
    u8::from(info.provenance().is_some())
}

/// Verify a file at path and return a ManifestStore report
///
/// # Errors
/// Returns an error field if there were errors
///
/// # Safety
/// Reads from a null terminated C string
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_verify_from_file(path: *const c_char) -> *mut c_char {
    result_to_c_string(verify_from_file_json(&from_c_str(path)))
}

/// Return an IngredientJson struct from the file at path
///
/// # Errors
/// Returns an error field if there were errors
///
/// # Safety
/// Reads from null terminated C strings
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_ingredient_from_file(
    path: *const c_char,
    data_dir: *const c_char,
    _flags: u8,
) -> *mut c_char {
    // convert C pointers into Rust
    let path = from_c_str(path);
    let data_dir = from_c_str(data_dir);

    let response = ingredient_from_file_json(&path, &data_dir); //, IngredientFlags::from_bits_truncate(flags));

    result_to_c_string(response)
}

#[repr(C)]
pub struct SignerInfoC {
    pub signcert: *const c_char,
    pub pkey: *const c_char,
    pub alg: *const c_char,
    pub tsa_url: *const c_char,
}

/// Add a signed manifest to the file at path using auth_token
/// If cloud is true, upload the manifest to the cloud
///
/// # Errors
/// Returns an error field if there were errors
///
/// # Safety
/// Reads from null terminated C strings
/// The returned value MUST be released by calling release_string
/// and it is no longer valid after that call.
#[no_mangle]
pub unsafe extern "C" fn c2pa_add_manifest_to_file(
    source_path: *const c_char,
    dest_path: *const c_char,
    manifest: *const c_char,
    signer_info: SignerInfoC,
    side_car: bool,
    remote_url: *const c_char,
) -> *mut c_char {
    // convert C pointers into Rust
    let source_path = from_c_str(source_path);
    let dest_path = from_c_str(dest_path);
    let manifest = from_c_str(manifest);
    let remote_url = if remote_url.is_null() {
        Some(from_c_str(remote_url))
    } else {
        None
    };
    let signer_info = SignerInfo {
        signcert: from_c_str(signer_info.signcert).into_bytes(),
        pkey: from_c_str(signer_info.pkey).into_bytes(),
        alg: from_c_str(signer_info.alg),
        tsa_url: if signer_info.tsa_url.is_null() {
            None
        } else {
            Some(from_c_str(signer_info.tsa_url))
        },
    };
    // Read manifest from JSON and then sign and write it
    let response = add_manifest_to_file_json(
        &source_path,
        &dest_path,
        &manifest,
        signer_info,
        side_car,
        remote_url,
    );

    result_to_c_string(response)
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
    let _release = CString::from_raw(s);
}

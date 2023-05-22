// ADOBE CONFIDENTIAL
// Copyright 2023 Adobe
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Adobe and its suppliers, if any. The intellectual
// and technical concepts contained herein are proprietary to Adobe
// and its suppliers and are protected by all applicable intellectual
// property laws, including trade secret and copyright laws.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Adobe.

/// This module exports a C2PA library
mod c_api;
mod json_api;
mod response;
mod signer_info;

pub use c2pa::{Error, Result, Signer};
pub use json_api::*;
pub use signer_info::SignerInfo;

uniffi::include_scaffolding!("c2pa_uniffi");

/// return the version of the c2pa SDK used in this library
fn sdk_version() -> String {
    String::from(c2pa::VERSION)
}

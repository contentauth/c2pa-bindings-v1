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

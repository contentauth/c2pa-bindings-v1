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

mod c_api;
/// This module exports a C2PA library
mod error;
mod ingredient_builder;
mod manifest_builder;
mod manifest_store_reader;
mod stream;

use c2pa::{jumbf_io::get_supported_types, Error as c2paError, Result as c2paResult, Signer};

pub use error::{C2paError, Result};

pub use c_api::C2paReader;
pub use ingredient_builder::IngredientBuilder;
pub use manifest_builder::ManifestBuilder;
pub use manifest_store_reader::ManifestStoreReader;
pub use stream::{ReadStream, SeekMode, StreamError, StreamResult};

uniffi::include_scaffolding!("c2pa_uniffi");

/// Returns the version of the C2PA library
pub fn version() -> String {
    format!(
        "{}/{} {}/{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        c2pa::NAME,
        c2pa::VERSION
    )
}

/// Returns a list of supported file extensions
pub fn supported_extensions() -> Vec<String> {
    let mut formats = get_supported_types()
        .iter()
        .filter(|t| !t.contains('/'))
        .map(|t| t.to_string())
        .collect::<Vec<_>>();
    formats.sort();
    formats
}

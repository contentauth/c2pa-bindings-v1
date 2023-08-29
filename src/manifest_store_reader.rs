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

use crate::{
    error::{C2paError, Result},
    StreamError,
};
use c2pa::ManifestStore;
use std::io::{Read, Seek};
use std::sync::RwLock;

struct ReaderSettings {}

pub struct ManifestStoreReader {
    _settings: ReaderSettings,
    store: RwLock<ManifestStore>,
}

impl ManifestStoreReader {
    pub fn new() -> Self {
        Self {
            _settings: ReaderSettings {},
            store: RwLock::new(ManifestStore::new()),
        }
    }

    pub fn read(&self, format: &str, mut stream: impl Read + Seek) -> Result<String> {
        // todo: use ManifestStore::from_stream, when it exists
        let mut bytes = Vec::new();
        let _len = stream.read_to_end(&mut bytes).map_err(|e| {
            C2paError::Stream(StreamError::Other {
                reason: e.to_string(),
            })
        })?;
        let store = ManifestStore::from_bytes(format, &bytes, true).map_err(C2paError::Sdk)?;
        let json = store.to_string();
        if let Ok(mut st) = self.store.try_write() {
            *st = store;
        } else {
            return Err(C2paError::RwLock);
        };
        Ok(json)
    }

    pub fn json(&self) -> Result<String> {
        self.store
            .try_read()
            .map(|store| (*store).to_string())
            .map_err(|_| C2paError::RwLock)
    }

    pub fn resource(&self, manifest: &str, id: &str) -> Option<Vec<u8>> {
        if let Ok(store) = self.store.try_read() {
            return store.manifests().get(manifest).and_then(|manifest| {
                match manifest.resources().get(id) {
                    Ok(r) => Some(r.into_owned()),
                    Err(_) => None,
                }
            });
        } else {
            None
        }
    }
}

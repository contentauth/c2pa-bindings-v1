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

use crate::error::{C2paError, Result};

use c2pa::Manifest;
use std::{
    io::{Read, Seek, Write},
    sync::RwLock,
};

//pub struct Signer {}

pub struct ManifestBuilderSettings {}

pub struct ManifestBuilder {
    manifest: RwLock<Manifest>,
}

impl ManifestBuilder {
    pub fn new(_settings: &ManifestBuilderSettings) -> Self {
        Self {
            manifest: RwLock::new(Manifest::new("test")),
        }
    }

    pub fn from_json(&self, json: &str) -> Result<()> {
        let manifest = c2pa::Manifest::from_json(json).map_err(C2paError::Sdk)?;
        if let Ok(mut m) = self.manifest.try_write() {
            *m = manifest;
        } else {
            return Err(C2paError::RwLock);
        };
        Ok(())
    }

    fn _set_format(&mut self, _format: &str) -> &mut Self {
        self
    }

    fn _set_title(&mut self, _title: &str) -> &mut Self {
        self
    }

    fn _set_remote_url(&mut self, _url: &str, _remote_only: bool) -> &mut Self {
        self
    }

    pub fn add_resource(&mut self, _id: &str, _resource: &[u8]) -> Result<&Self> {
        Ok(self)
    }

    pub fn sign(
        &mut self,
        input: impl Read + Seek,
        _output: Option<impl Read + Write + Seek>,
        _c2pa_data: Option<impl Write>,
    ) -> Result<()> {
        let _input = std::io::Cursor::new(input);
        if let Ok(_manifest) = self.manifest.try_read() {
            // let result = manifest.embed_stream(manifest.format(), input, Signer).map_err(C2paError::Sdk)?;
            // return store
            //     .manifests()
            //     .get(manifest)
            //     .and_then(|manifest|
            //         match manifest.resources().get(id) {
            //             Ok(r) => {
            //                 Some(r.into_owned())
            //             }
            //             Err(_) => {
            //                 None
            //             }
            //         })
            Ok(())
        } else {
            Err(C2paError::RwLock)
        }
    }
}

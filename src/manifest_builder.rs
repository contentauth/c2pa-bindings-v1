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

struct CAIReadWrapper<'a> {
    pub reader: &'a mut dyn c2pa::CAIRead,
}

impl Read for CAIReadWrapper<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Seek for CAIReadWrapper<'_> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}


//pub struct Signer {}

pub struct ManifestBuilderSettings {
    pub generator: String
}

pub struct ManifestBuilder {
    manifest: RwLock<Manifest>,
}

impl ManifestBuilder {
    pub fn new(settings: &ManifestBuilderSettings) -> Self {
        Self {
            manifest: RwLock::new(Manifest::new(settings.generator.clone())),
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
        &self,
        mut input: impl c2pa::CAIRead,
        _output: Option<impl Read + Write + Seek>,
        //_c2pa_data: Option<impl Write>,
    ) -> Result<()> {
        //let _input = std::io::Cursor::new(input);
        let size = input.seek(std::io::SeekFrom::End(0)).expect("SeekTest failed");
        println!("Stream size = {}", size);
        if let Ok(mut manifest) = self.manifest.try_write() {
            let format = manifest.format().to_string();
            let signer = c2pa::create_signer::from_files("tests/fixtures/es256_certs.pem", "tests/fixtures/es256_private.key", c2pa::SigningAlg::Es256, None).map_err(C2paError::Sdk)?;
            let result = manifest.embed_stream(&format, &mut input,  &*signer).map_err(C2paError::Sdk).expect("Failed to embed stream");
            if let Some(mut output) = _output {
                output.write_all(&result).map_err(C2paError::Io)?;
            };
            Ok(())
        } else {
            Err(C2paError::RwLock)
        }
    }
}

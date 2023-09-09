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
    stream::{Stream, StreamAdapter},
};
use c2pa::Manifest;
use std::{
    io::{Read, Seek, Write},
    sync::RwLock,
};

//pub struct Signer {}

pub struct ManifestBuilderSettings {
    pub generator: String,
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

    pub fn sign_stream(
        &self,
        input: Box<dyn Stream>,
        output: Option<impl Read + Write + Seek>,
        //_c2pa_data: Option<impl Write>,
    ) -> Result<()> {
        let input_ref = &*input as *const dyn Stream as *mut dyn Stream;
        let input_ref = unsafe { &mut *input_ref };
        let mut input = StreamAdapter::from_stream(input_ref);
        self.sign(&mut input, output)
    }

    pub fn sign(
        &self,
        input: &mut StreamAdapter,
        output: Option<impl Read + Write + Seek>,
        //_c2pa_data: Option<impl Write>,
    ) -> Result<()> {
        if let Ok(mut manifest) = self.manifest.try_write() {
            let format = manifest.format().to_string();
            //println!("Format = {}", format);
            let signer = c2pa::create_signer::from_files(
                "tests/fixtures/es256_certs.pem",
                "tests/fixtures/es256_private.key",
                c2pa::SigningAlg::Es256,
                None,
            )
            .map_err(C2paError::Sdk)?;
            let result = manifest
                .embed_stream(&format, input, &*signer)
                .map_err(C2paError::Sdk)
                .expect("Failed to embed stream");
            if let Some(mut output) = output {
                output.write_all(&result).map_err(C2paError::Io)?;
            };
            Ok(())
        } else {
            Err(C2paError::RwLock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_stream::TestStream;
    use std::io::Cursor;

    const MANIFEST_JSON: &str = r#"
    {
        "claim_generator": "test_generator",
        "format": "image/jpeg",
        "title": "test_title"
    }
    "#;

    const IMAGE: &'static [u8] = include_bytes!("../tests/fixtures/A.jpg");

    #[test]
    fn test_manifest_builder() {
        let settings = ManifestBuilderSettings {
            generator: "test".to_string(),
        };
        let builder = ManifestBuilder::new(&settings);
        builder
            .from_json(MANIFEST_JSON)
            .expect("Failed to load manifest Json");
        let mut input = TestStream::from_memory(IMAGE.to_vec());
        let mut input = StreamAdapter::from_stream(&mut input);
        let mut output = Cursor::new(Vec::new());
        builder
            .sign(&mut input, Some(&mut output))
            .expect("Failed to sign");
        let result = output.into_inner();
        assert_eq!(result.len(), 128977);
    }

    #[test]
    fn test_manifest_builder_with_stream() {
        let settings = ManifestBuilderSettings {
            generator: "test".to_string(),
        };
        let builder = ManifestBuilder::new(&settings);
        builder
            .from_json(MANIFEST_JSON)
            .expect("Failed to load manifest Json");
        let input = TestStream::from_memory(IMAGE.to_vec());
        let mut output = Cursor::new(Vec::new());
        builder
            .sign_stream(Box::new(input), Some(&mut output))
            .expect("Failed to sign");
        let result = output.into_inner();
        // std::fs::write("target/manifest_builder.jpg", &result).expect("Failed to write test file");
        assert_eq!(result.len(), 128977);
    }
}

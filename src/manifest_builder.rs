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

use std::{collections::HashMap, sync::RwLock};

use c2pa::{CAIRead, CAIReadWrite, Manifest, Signer};

use crate::{
    error::{C2paError, Result},
    stream::{Stream, StreamAdapter},
    C2paSigner,
};

pub struct ManifestBuilderSettings {
    pub generator: String,
}

trait StreamResolver: Send + Sync {
    fn stream_for_id(&mut self, id: &str) -> Option<&mut Box<dyn Stream>>;
}

struct StreamTable {
    streams: HashMap<String, Box<dyn Stream>>,
}

impl StreamResolver for StreamTable {
    fn stream_for_id(&mut self, id: &str) -> Option<&mut Box<dyn Stream>> {
        self.streams.get_mut(id)
    }
}

pub struct ManifestBuilder {
    manifest: RwLock<Manifest>,
    _resolvers: Vec<Box<dyn StreamResolver>>,
}

impl ManifestBuilder {
    pub fn new(settings: &ManifestBuilderSettings) -> Self {
        Self {
            manifest: RwLock::new(Manifest::new(settings.generator.clone())),
            _resolvers: Vec::new(),
        }
    }

    fn unlock_write(&self) -> Result<std::sync::RwLockWriteGuard<Manifest>> {
        self.manifest.try_write().map_err(|_| C2paError::RwLock)
    }

    pub fn from_json(&self, json: &str) -> Result<()> {
        *self.unlock_write()? = c2pa::Manifest::from_json(json).map_err(C2paError::Sdk)?;
        Ok(())
    }

    pub fn set_format(&mut self, format: &str) -> Result<&mut Self> {
        self.unlock_write()?.set_format(format);
        Ok(self)
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

    pub fn add_resource_stream(&mut self, _id: &str, _stream: Box<dyn Stream>) -> Result<&Self> {
        Ok(self)
    }

    pub fn sign_stream(
        &self,
        signer: &C2paSigner,
        input_mut: &dyn Stream,
        output_mut: &dyn Stream,
    ) -> Result<Vec<u8>> {
        let mut input = StreamAdapter::from(input_mut);
        let mut output = StreamAdapter::from(output_mut);
        self.sign(signer, &mut input, &mut output)
    }

    pub fn sign(
        &self,
        signer: &dyn Signer,
        input: &mut dyn CAIRead,
        output: &mut dyn CAIReadWrite,
    ) -> Result<Vec<u8>> {
        let mut manifest = self.unlock_write()?;
        let format = manifest.format().to_string();
        manifest
            .embed_to_stream( &format, input, output, signer)
            .map_err(C2paError::Sdk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{signer::C2paSigner, test_signer::TestSigner, test_stream::TestStream, SeekMode};
    use std::io::Seek;

    const MANIFEST_JSON: &str = r#"
    {
        "claim_generator": "test_generator",
        "format": "image/jpeg",
        "title": "test_title",
        "thumbnail": {
            "format": "image/jpeg",
            "identifier": "thumbnail"
        }
    }
    "#;

    const IMAGE: &'static [u8] = include_bytes!("../tests/fixtures/A.jpg");
    const CERTS: &'static [u8] = include_bytes!("../tests/fixtures/ps256.pub");
    const P_KEY: &'static [u8] = include_bytes!("../tests/fixtures/ps256.pem");

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
        let mut input = StreamAdapter::from_stream_mut(&mut input);
        //let mut output = Cursor::new(Vec::new());
        let mut output = TestStream::new();
        let mut output = StreamAdapter::from_stream_mut(&mut output);
        let signer = c2pa::create_signer::from_keys(CERTS, P_KEY, c2pa::SigningAlg::Ps256, None)
            .map_err(C2paError::Sdk)
            .expect("Failed to create signer");
        builder
            .sign(&*signer, &mut input, &mut output)
            .expect("Failed to sign");
        let len = output.seek(std::io::SeekFrom::End(0)).unwrap();
        assert_eq!(len, 134165);
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
        let mut input = TestStream::from_memory(IMAGE.to_vec());
        let mut output = TestStream::new();
        let test_signer = Box::new(TestSigner::new());
        let config = test_signer.config();
        let signer = C2paSigner::new(test_signer);
        signer.configure(&config).expect("Signer config failed");
        builder
            .sign_stream(&signer, &mut input, &mut output)
            .expect("Failed to sign");
        let len = output.seek_stream(0, SeekMode::End).unwrap();
        assert_eq!(len, 143141);
    }
}

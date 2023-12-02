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

use std::{io::Cursor, sync::RwLock};

use c2pa::ManifestStore;

use crate::{C2paError, Result, Stream, StreamAdapter};

pub(crate) struct CAIReadWrapper<'a> {
    pub reader: &'a mut dyn c2pa::CAIRead,
}

impl std::io::Read for CAIReadWrapper<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl std::io::Seek for CAIReadWrapper<'_> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

struct ReaderSettings {}

/// The ManifestStoreReader reads the manifest store from a stream and then
/// provides access to the store via the json() and resource() methods.
pub struct ManifestStoreReader {
    _settings: ReaderSettings,
    store: RwLock<ManifestStore>,
}

impl ManifestStoreReader {
    /// Creates a new ManifestStoreReader
    /// # Returns
    /// * `ManifestStoreReader` - the new ManifestStoreReader
    ///
    pub fn new() -> Self {
        Self {
            _settings: ReaderSettings {},
            store: RwLock::new(ManifestStore::new()),
        }
    }

    /// Reads the manifest store from a stream
    /// # Arguments
    /// * `format` - the format of the manifest store
    /// * `stream` - the stream to read from
    /// # Returns
    /// * `Result<String>` - the json representation of the manifest store
    ///    or an error
    ///
    pub fn read_stream(&self, format: &str, stream: &dyn Stream) -> Result<String> {
        let mut stream = StreamAdapter::from(stream);
        self.read(format, &mut stream)
    }

    /// Reads the manifest store from a stream
    /// # Arguments
    /// * `format` - the format of the manifest store
    /// * `stream` - the stream to read from
    /// # Returns
    /// * `Result<String>` - the json representation of the manifest store
    ///    or an error
    ///
    pub fn read(&self, format: &str, stream: &mut dyn c2pa::CAIRead) -> Result<String> {
        // todo: use ManifestStore::f
        let mut bytes = Vec::new();
        stream
            .seek(std::io::SeekFrom::End(0))
            .map_err(C2paError::from)?;
        stream
            .seek(std::io::SeekFrom::Start(0))
            .map_err(C2paError::from)?;
        let _len = stream.read_to_end(&mut bytes).map_err(C2paError::from)?;
        let bytes = CAIReadWrapper {
            reader: &mut Cursor::new(bytes),
        };
        let stream = bytes.reader;
        let store = ManifestStore::from_stream(format, stream, true).map_err(C2paError::from)?;
        let json = store.to_string();
        if let Ok(mut st) = self.store.try_write() {
            *st = store;
        } else {
            return Err(C2paError::RwLock);
        };
        Ok(json)
    }

    /// returns a json representation of the manifest store
    /// # Returns
    /// * `Result<String>` - the json representation of the manifest store
    ///     or an error
    ///
    pub fn json(&self) -> Result<String> {
        self.store
            .try_read()
            .map(|store| (*store).to_string())
            .map_err(|_e| C2paError::Other("RWLock".to_string()))
    }

    /// returns a resource from the manifest store
    /// # Arguments
    /// * `manifest` - the manifest id
    /// * `id` - the resource id
    /// # Returns
    /// * `Option<Vec<u8>>` - the resource bytes
    ///
    pub fn resource(&self, manifest: &str, id: &str) -> Result<Vec<u8>> {
        if let Ok(store) = self.store.try_read() {
            match store.manifests().get(manifest) {
                Some(manifest) => match manifest.resources().get(id) {
                    Ok(r) => Ok(r.into_owned()),
                    Err(e) => Err(C2paError::from(e)),
                },
                None => Err(C2paError::from(c2pa::Error::ResourceNotFound(
                    manifest.to_string(),
                ))),
            }
        } else {
            Err(C2paError::RwLock)
        }
    }

    pub fn resource_write_stream(
        &self,
        manifest_label: &str,
        id: &str,
        stream: &dyn Stream,
    ) -> Result<()> {
        let mut stream = StreamAdapter::from(stream);
        self.resource_write(manifest_label, id, &mut stream)
    }

    /// writes a resource from the manifest store to the stream
    /// # Arguments
    /// * `manifest` - the manifest id
    /// * `id` - the resource id
    /// * `stream` - the stream to write to
    /// # Returns
    /// * `Result<()>` - Ok or an error
    ///
    pub fn resource_write(
        &self,
        manifest_label: &str,
        id: &str,
        stream: &mut dyn c2pa::CAIReadWrite,
    ) -> Result<()> {
        self.resource(manifest_label, id)
            .and_then(|bytes| stream.write_all(&bytes).map_err(C2paError::from))
    }
}

impl Default for ManifestStoreReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::test_stream::TestStream;

    const IMAGE: &'static [u8] = include_bytes!("../tests/fixtures/C.jpg");

    #[test]
    fn test_manifest_store_reader() {
        let mut input = TestStream::from_memory(IMAGE.to_vec());
        let reader = ManifestStoreReader::new();
        let json = reader.read_stream("image/jpeg", &mut input).unwrap();
        println!("Json = {}", json);
        assert!(json.contains("\"format\": \"image/jpeg\""));
        assert!(json.contains("\"title\": \"C.jpg\""));
        assert!(!json.contains("\"validation_status\":"));
    }
}

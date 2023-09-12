# Copyright 2023 Adobe. All rights reserved.
# This file is licensed to you under the Apache License,
# Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
# or the MIT license (http://opensource.org/licenses/MIT),
# at your option.

# Unless required by applicable law or agreed to in writing,
# this software is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
# implied. See the LICENSE-MIT and LICENSE-APACHE files for the
# specific language governing permissions and limitations under
# each license.

import json
import os
import sys
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)

import c2pa;

ManifestStoreReader = c2pa.ManifestStoreReader

# Implements a C2paStream given a stream handle
class C2paStream(c2pa.Stream):
    def __init__(self, stream):
        self.stream = stream
    
    def read_stream(self, length: int) -> bytes:   
        return self.stream.read(length)

    def seek_stream(self, pos: int, mode: c2pa.SeekMode) -> int:
        whence = 0
        if mode is c2pa.SeekMode.CURRENT:
            whence = 1
        elif mode is c2pa.SeekMode.END:
            whence = 2
        #print("Seeking to " + str(pos) + " with whence " + str(whence))
        return self.stream.seek(pos, whence)

    def write_stream(self, data: str) -> int:
        return self.stream.write(data)

    def flush_stream(self) -> None:
        self.stream.flush()

    # A shortcut method to open a C2paStream from a path/mode
    def open_file(path: str, mode: str) -> c2pa.Stream:
        return C2paStream(open(path, mode))

class LocalSigner(c2pa.SignerCallback):

    def __init__(self, config, private_key):
        self.config = config
        self.private_key = private_key

    def sign(self, data: bytes) -> bytes:
        return c2pa.local_sign(data, self.private_key)

class Manifest:
    def __init__(self, title, format, claim_generator, thumbnail, ingredients, assertions, sig_info=None):
        self.title = title
        self.format = format
        self.claim_generator = claim_generator
        self.thumbnail = thumbnail
        self.ingredients = ingredients
        self.assertions = assertions
        self.signature_info = sig_info

class ManifestStore:
    def __init__(self, activeManifest, manifests, validationStatus=None):
        self.activeManifest = activeManifest
        self.manifests = manifests
        self.validationStatus = validationStatus
        
    def __str__(self):
        return json.dumps(dict(self), ensure_ascii=False)
    
    @staticmethod
    def from_json(json_str):
        json_dct = json.loads(json_str)
        manifests = {}
        for label, manifest in json_dct["manifests"].items():
            manifests[label] = Manifest(
                manifest.get("title"),
                manifest.get("format"),
                manifest.get("claim_generator"),
                manifest.get("thumbnail"),
                manifest.get("ingredients"),
                manifest.get("assertions"),
                manifest.get("signature_info")
            )

        return ManifestStore(json_dct['active_manifest'],
                manifests, json_dct.get('validation_status'))

__all__ = ["C2paStream", "Manifest", "ManifestStore", "ManifestStoreReader"]
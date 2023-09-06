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
        if mode == c2pa.SeekMode.Current:
            whence = 1
        elif mode == c2pa.SeekMode.End:
            whence = 2
        self.stream.seek(pos, whence)

    def write_stream(self, data: str) -> int:
        return self.stream.write(data)

    def flush_stream(self) -> None:
        self.stream.flush()

    # A shortcut method to open a C2paStream from a path/mode
    def open_file(path: str, mode: str) -> c2pa.Stream:
        return C2paStream(open(path, mode))

    

class Manifest:
    def __init__(self, title, format, claim_generator, thumbnail, ingredients, assertions):
        self.title = title
        self.format = format
        self.claim_generator = claim_generator
        self.thumbnail = thumbnail
        self.ingredients = ingredients
        self.assertions = assertions

class ManifestStore:
    def __init__(self, activeManifest, manifests):
        self.activeManifest = activeManifest
        self.manifests = manifests
        
    def __str__(self):
        return json.dumps(dict(self), ensure_ascii=False)
    
    @staticmethod
    def from_json(json_str):
        json_dct = json.loads(json_str)
        manifests = {}
        for label, manifest in json_dct["manifests"].items():
            manifests[label] = Manifest(
                manifest["title"],
                manifest["format"],
                manifest["claim_generator"],
                manifest["thumbnail"],
                manifest["ingredients"],
                manifest["assertions"]
            )

        return ManifestStore(json_dct['active_manifest'],
                manifests)

__all__ = ["C2paStream", "Manifest", "ManifestStore", "ManifestStoreReader"]
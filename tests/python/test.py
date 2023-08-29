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
# each license.import os

import sys
import os
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)

import c2pa_api

testFile = os.path.join(PROJECT_PATH,"tests","fixtures","C.jpg")
file = open(testFile, "rb") 
reader = c2pa_api.C2paReader(file)
manifestStore = c2pa_api.ManifestStoreReader()
json = manifestStore.read("image/jpeg",reader)
print(json)

manifest_store = c2pa_api.ManifestStore.from_json(json)

activeManifest = manifest_store.activeManifest
print(activeManifest)
if activeManifest: 
    manifest = manifest_store.manifests[activeManifest]
    thumb_id = manifest.thumbnail["identifier"]
    thumb = manifestStore.resource(activeManifest, thumb_id)
    print(len(thumb))



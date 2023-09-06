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
import json
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)

import c2pa_api

testFile = os.path.join(PROJECT_PATH,"tests","fixtures","C.jpg")
file = open(testFile, "rb") 
stream = c2pa_api.C2paStream(file)
manifestStore = c2pa_api.ManifestStoreReader()
report = manifestStore.read("image/jpeg",stream)
print(report)

manifest_store = c2pa_api.ManifestStore.from_json(report)

manifest_label = manifest_store.activeManifest
print(manifest_label)
if manifest_label: 
    manifest = manifest_store.manifests[manifest_label]
    thumb_id = manifest.thumbnail["identifier"]
    thumb = manifestStore.resource(manifest_label, thumb_id)
    print(len(thumb))
    # now write the thumbnail to a file
    thumb_file = os.path.join(PROJECT_PATH,"target","thumb_from_python.jpg")
    if os.path.exists(thumb_file):
        os.remove(thumb_file)
    #if not os.path.exists(os.path.dirname(thumb_file)):
    #    os.makedirs(os.path.dirname(thumb_file))
    thumbOut = c2pa_api.C2paStream.open_file(thumb_file, "wb")
    manifestStore.resource_write(manifest_label, thumb_id, thumbOut)
    if not os.path.exists(thumb_file):
        print("Failed to write thumbnail")
    else:
        print("Thumbnail written to " + thumb_file)

manifest = {
    "claim_generator": "python_test",
    "claim_generator_info": [{
        "name": "python_test",
        "version": "0.0.1",
    }],
    "ingredients": [],
    "assertions": [
        {   'label': 'stds.schema-org.CreativeWork',
            'data': {
                '@context': 'http://schema.org/',
                '@type': 'CreativeWork',
                'author': [
                    {   '@type': 'Person',
                        'name': 'Gavin Peacock'
                    }
                ]
            },
            'kind': 'Json'
        }
    ]
 }

settings = c2pa_api.c2pa.ManifestBuilderSettings("foo") #{ 'generator': "foo" }
manifest_builder = c2pa_api.c2pa.ManifestBuilder(settings)
manifest_builder.from_json(json.dumps(manifest))
manifest_builder.sign(stream, None)

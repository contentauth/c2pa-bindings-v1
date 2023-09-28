
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

import os
import sys
import json
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)

import c2pa;

testFile = os.path.join(PROJECT_PATH,"tests","fixtures","C.jpg")
pemFile = os.path.join(PROJECT_PATH,"tests","fixtures","es256_certs.pem")
keyFile = os.path.join(PROJECT_PATH,"tests","fixtures","es256_private.key")
testOutputFolder = os.path.join(PROJECT_PATH,"target","pytest")
testOutputFile = os.path.join(PROJECT_PATH,"target","test.jpg")

with open(pemFile,"rb") as f:
    test_pem = bytearray(f.read())

with open(keyFile,"rb") as f:
    test_key = bytearray(f.read())

try:
    report = c2pa.verify_from_file_json(testFile)
except Exception as err:
    sys.exit(err)

print(report)  

try:
    report = c2pa.ingredient_from_file_json(testFile, testOutputFolder)
except Exception as err:
    sys.exit(err)

print(report)

generator = "python_test/0.1"
author = "Joe Blogs"

manifest = json.dumps({
    "claim_generator": generator,
    "ingredients": [],
    "assertions": [
        {   'label': 'stds.schema-org.CreativeWork',
            'data': {
                '@context': 'http://schema.org/',
                '@type': 'CreativeWork',
                'author': [
                    {   '@type': 'Person',
                        'name': author
                    }
                ]
            },
            'kind': 'Json'
        }
    ]
 })

try:
    sign_info = c2pa.SignerInfo(test_pem, test_key, "es256", "http://timestamp.digicert.com")
    result = c2pa.add_manifest_to_file_json(testFile, testOutputFile, manifest, sign_info, False, None)
except Exception as err:
    sys.exit(err)

print("successfully added manifest to file")

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
# each license.import unittest

import unittest
from unittest.mock import mock_open, patch
import c2pa_api
from c2pa_api import c2pa
import os
PROJECT_PATH = os.getcwd()

testPath = os.path.join(PROJECT_PATH, "tests", "fixtures", "C.jpg")

class TestC2paSdk(unittest.TestCase):

    def test_version(self):
        self.assertIn("c2pa-rs/",c2pa_api.c2pa.version())

    def test_supported_extensions(self):
        self.assertIn("jpeg",c2pa_api.c2pa.supported_extensions())


class TestManifestStoreReader(unittest.TestCase):

    def test_normal_read(self):
        with open(testPath, "rb") as file:
            reader = c2pa_api.C2paStream(file)
            manifestStore = c2pa_api.ManifestStoreReader()
            json = manifestStore.read("image/jpeg",reader)
            self.assertIn("C.jpg", json)

    def test_normal_read_and_parse(self):
        with open(testPath, "rb") as file:
            reader = c2pa_api.C2paStream(file)
            manifestStore = c2pa_api.ManifestStoreReader()
            manifestStore.read("image/jpeg",reader)
            json = manifestStore.json()
            manifest_store = c2pa_api.ManifestStore.from_json(json)
            title= manifest_store.manifests[manifest_store.activeManifest].title
            self.assertEqual(title, "C.jpg")

    def test_json_decode_err(self):
        with self.assertRaises(c2pa_api.json.decoder.JSONDecodeError):
            manifest_store = c2pa_api.ManifestStore.from_json("foo")

    def test_reader_bad_format(self):
        with self.assertRaises(c2pa_api.c2pa.StreamError.Other):
            with open(testPath, "rb") as file:
                reader = c2pa_api.C2paStream(file)
                manifestStore = c2pa_api.ManifestStoreReader()
                json = manifestStore.read("badFormat",reader)

if __name__ == '__main__':
    unittest.main()
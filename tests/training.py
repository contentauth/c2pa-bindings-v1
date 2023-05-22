import json
import os
import sys
# load the c2pa_uniffi module from wherever you keep it
# In this case, it's in the target/python folder
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)
testFile = os.path.join(PROJECT_PATH,"tests","fixtures","C.jpg")

import c2pa_uniffi as c2pa;

# convert the manifestStore to a python object
class ManifestStore:
    def __init__(self, **kwargs):
        for key, value in kwargs.items():
            if isinstance(value, dict):
                self.__dict__[key] = ManifestStore(**value)
            else:
                self.__dict__[key] = value

try:
    report = c2pa.verify_from_file_json(testFile)
except Exception as err:
    sys.exit(err)

jsonStore = json.loads(report)

manifestStore = ManifestStore(**jsonStore)

activeManifest = manifestStore.active_manifest
print(activeManifest)

#manifest = manifestStore.manifests[manifestStore.active_manifest]
manifest = jsonStore["manifests"][jsonStore["active_manifest"]]

print(manifest.title, manifest.format, manifest.claim_generator)
for assertion in manifest.assertions:
    if assertion.label == "c2pa.training-mining":
        entries = assertion.data.entries
        if entries.c2pa.ai_training == "notAllowed":
            print("not allowed")
        print(entries.c2pa.ai_training)

manifest = jsonStore["manifests"][jsonStore["active_manifest"]]

print(manifest["title"], manifest["format"], manifest["claim_generator"])

allowed = True # opt out model
assertions = manifest["assertions"]
for assertion in assertions:
    if assertion["label"] == "c2pa.training-mining":
        allowed = assertion["data"]["entries"]["c2pa.ai_training"] == "notAllowed" 
    print(assertion)

import json
import os
import sys
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

#options = c2pa.IngredientOpts(True, True)
try:
    report = c2pa.ingredient_from_file_json(testFile, testOutputFolder)
except Exception as err:
    sys.exit(err)

print(report)


with open(testFile, mode="rb") as test_file:
    test_bytes = test_file.read()

# test reading a manifestStore structure
#try: 
#    manifestStore = c2pa.verify_from_bytes(test_bytes,"image/jpeg")
#except Exception as err:
#    sys.exit(err)

# get the active manifest and display info about it
#manifest = manifestStore.manifests[manifestStore.active_manifest]
#print(manifest.title, manifest.format, manifest.claim_generator)
#print(manifest.ingredients)
#for a in manifest.assertions:
#    print(a)

sign_info = c2pa.SignerInfo(test_pem, test_key, "es256", "http://timestamp.digicert.com")

# print(sign_info)
generator = "python_test/0.1"
author = "Gavin Peacock"

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
    result = c2pa.add_manifest_to_file_json(testFile, testOutputFile, manifest, sign_info, False, None)
except Exception as err:
    sys.exit(err)

print("successfully added manifest to file")


#print(m.claim_generator())
#print(m.to_string())

generator = "python_test"
author = "Gavin Peacock"

manifest = {
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
 }


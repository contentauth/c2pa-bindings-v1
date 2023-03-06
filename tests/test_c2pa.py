import json
import os
import sys
PROJECT_PATH = os.getcwd()
SOURCE_PATH = os.path.join(
    PROJECT_PATH,"target","python"
)
sys.path.append(SOURCE_PATH)

import c2pa_uniffi as c2pa;

testFile = os.path.join(PROJECT_PATH,"tests","fixtures","C.jpg")
testOutputFolder = os.path.join(PROJECT_PATH,"target","pytest")

report = c2pa.c2pa_verify_from_file(testFile)
result = json.loads(report)
if 'error' in result:
    sys.exit("verify error: " + json.dumps(result['error']))
print(report)

try:
    report = c2pa.verify_from_file(testFile)
except Exception as err:
    sys.exit(err)
print(report)  

#options = c2pa.IngredientOpts(True, True)
report = c2pa.c2pa_ingredient_from_file(testFile, testOutputFolder)
print(report)

manifest = {
    "claim_generator": "python_test",
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

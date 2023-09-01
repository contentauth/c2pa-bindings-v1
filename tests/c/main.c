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
// each license.#include "c2pa.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "c2pa.h"

const char * asset_path = "tests/fixtures/A.jpg";

ssize_t reader(size_t context, uint8_t *data, size_t len) {
    printf("reader: context = %zu, data = %p, len = %zu\n", context, data, len);
    return fread(data, 1, len, (FILE*)context);
}

int seeker(size_t context,long int offset, int whence) {
    printf("seeker: context = %zu, offset = %ld, whence = %d\n", context, offset, whence);
    return fseek((FILE*)context, offset, whence);
}

int writer(size_t context, uint8_t *data, size_t len) {
    printf("writer: context = %zu, data = %p, len = %zu\n", context, data, len);
    return fwrite(data, 1, len, (FILE*)context);
}

C2paStream* create_stream(FILE *file) {
    if (file != NULL) {
       return c2pa_create_stream(file, reader, seeker, writer);
    }
    return NULL;
}

// Function to find the value associated with a key in a JSON string
char* findValueByKey(const char* json, const char* key) {
    const char* keyStart = strstr(json, key);

    if (keyStart == NULL) {
        return NULL;  // Key not found
    }

    const char* valueStart = strchr(keyStart, ':');
    if (valueStart == NULL) {
        return NULL;  // Malformed JSON
    }

    // Move past the ':' and whitespace
    valueStart++;
    while (*valueStart == ' ' || *valueStart == '\t' || *valueStart == '\n' || *valueStart == '\r') {
        valueStart++;
    }

    if (*valueStart == '"') {
        // String value
        const char* valueEnd = strchr(valueStart + 1, '"');
        if (valueEnd == NULL) {
            return NULL;  // Malformed JSON
        }
        int valueLength = valueEnd - valueStart - 1;
        char* result = (char*)malloc(valueLength + 1);
        strncpy(result, valueStart + 1, valueLength);
        result[valueLength] = '\0';
        return result;
    } else {
        // Numeric or other value
        const char* valueEnd = valueStart;
        while (*valueEnd != ',' && *valueEnd != '}' && *valueEnd != ']' && *valueEnd != '\0') {
            valueEnd++;
        }
        int valueLength = valueEnd - valueStart;
        char* result = (char*)malloc(valueLength + 1);
        strncpy(result, valueStart, valueLength);
        result[valueLength] = '\0';
        return result;
    }
}

int main(void) {
    //FILE *file;

    char *version = c2pa_version();
    printf("version = %s\n", version);
    c2pa_release_string(version);

    char *extensions = c2pa_supported_extensions();
    printf("supported extensions = %s\n", extensions);
    c2pa_release_string(extensions);

	//Open file
    FILE *input_file = fopen("tests/fixtures/C.jpg", "rb");
    if (!input_file)
	{
		fprintf(stderr, "Unable to open file");
		return 1;
	}
    //printf("input_file = %zu\n", input_file);
	C2paStream* input_stream = c2pa_create_stream(input_file, reader, seeker, writer);
    if (input_stream == NULL) {
        printf("error creating stream = %s\n", c2pa_error());
        return 1;
    }

    // char* result = c2pa_verify_stream(*file);
    // if (result == NULL) {
    //     result = c2pa_error();
    // }
    // printf("result = %s\n", result);

    void* manifest_reader = c2pa_manifest_reader_new();
    if (manifest_reader == NULL) {
        printf("manifest new err = %s\n", c2pa_error());
        return 1;
    }
    char* result = c2pa_manifest_reader_read(&manifest_reader, "image/jpeg", *input_stream);
    if (result == NULL) {
        printf("manifest read err = %s\n", c2pa_error());
        return 1;
    }
    c2pa_release_stream(input_stream);

    // display the manifest store json
    printf("manifest json = %s\n", result);

    // We should be using a JSON parser here, but this is a quick and dirty way to get the values we need
    // find the active manifest label and the identifier for the manifest thumbnail
    char* manifest_label = findValueByKey(result, "active_manifest");
    if (manifest_label== NULL) {
        printf("no active manifest");
        return 1;
    }
    // we should fetch the active manifest and retrieve the identifier from the thumbnail in that manifest 
    char *id = findValueByKey(result, "identifier");
    if (id == NULL) {
        printf("identifier not found\n");
        return 1;
    }
    printf("Searching for thumbnail %s : %s\n", manifest_label, id);

	//Open a file to write the thumbnail to
	FILE* thumb_file = fopen("target/thumb_c.jpg", "wb");
	if (!thumb_file)
	{
		fprintf(stderr, "Unable to create thumb file");
		return 1;
	}
    // create a c2pa stream for the thumbnail
    C2paStream* thumb_stream = c2pa_create_stream(thumb_file, reader, seeker, writer);
    if (thumb_stream == NULL) {
        printf("error creating thumb stream = %s\n", c2pa_error());
        return 1;
    }

    // write the thumbnail resource to the stream
    c2pa_manifest_reader_resource(&manifest_reader, manifest_label, id, *thumb_stream);

    free(manifest_label);
    free(id);

    c2pa_release_string(result);
    c2pa_release_stream(thumb_stream);
    fclose(input_file);
    fclose(thumb_file);
}
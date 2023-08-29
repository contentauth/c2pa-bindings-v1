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
    //printf("reader: context = %zu, data = %p, len = %zu\n", context, data, len);
    return fread(data, 1, len, (FILE*)context);
}

int seeker(size_t context,long int offset, int whence) {
    // printf("seeker: context = %zu, offset = %ld, whence = %d\n", context, offset, whence);
    return fseek((FILE*)context, offset, whence);
}

int main(void) {
    FILE *file;

    char *version = c2pa_version();
    printf("version = %s\n", version);
    c2pa_release_string(version);

    char *extensions = c2pa_supported_extensions();
    printf("supported extensions = %s\n", extensions);
    c2pa_release_string(extensions);

	//Open file
	file = fopen("tests/fixtures/C.jpg", "rb");
	if (!file)
	{
		fprintf(stderr, "Unable to open file");
		return 1;
	}
    C2paReader* c2pa_reader = c2pa_create_reader(file, reader, seeker);
    char* result = c2pa_verify_stream(*c2pa_reader);
    printf("result = %s\n", result);
    c2pa_release_string(result);
    //c2pa_release_reader(c2pa_reader);
}
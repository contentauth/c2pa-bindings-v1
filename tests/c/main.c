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
#include <errno.h>
#include "c2pa.h"

const char * asset_path = "tests/fixtures/A.jpg";


const char* manifest_json = "{\
    \"claim_generator\":\"c-test\",\
    \"title\":\"C Test Image\",\
    \"format\":\"image/jpeg\",\
    \"ingredients\":[], \
    \"assertions\":[] \
}";

char* load_file(const char* filename) {
    char* buffer = NULL;
    long file_size;
    FILE* fp = fopen(filename, "rb");  // Open file in binary mode

    if (fp != NULL) {
        // Determine file size
        fseek(fp, 0L, SEEK_END);
        file_size = ftell(fp);
        rewind(fp);

        // Allocate buffer
        buffer = (char*) malloc(file_size + 1);
        if (buffer != NULL) {
            // Read file into buffer
            fread(buffer, 1, file_size, fp);
            buffer[file_size] = '\0';  // Add null terminator
        }
        fclose(fp);
    }
    return buffer;
}

int save_file(char* filename, uint8_t*data, size_t len) {
    FILE* fp = fopen(filename, "wb");  
    int bytes_written = -1; 
    // Open file in binary mode
    if (fp != NULL) {
        bytes_written = fwrite(data, len, 1, fp);
        fclose(fp);
    }
    return bytes_written;
}

ssize_t reader(size_t context, uint8_t *data, size_t len) {
    //printf("reader: context = %0lx, data = %p, len = %zu\n", context, data, len);
    return fread(data, 1, len, (FILE*)context);
}

int seeker(size_t context,long int offset, SeekMode whence) {
    switch (whence) {
        case Start:
            whence = SEEK_SET;
            break;
        case End:
            whence = SEEK_END;
            break;
        case Current:
            whence = SEEK_CUR;
            break;
    };
    //printf("seeker: context = %0lx, offset = %ld, whence = %d\n", context, offset, whence);
    long int result = fseek((FILE*)context, offset, whence);
    //printf("seeker: result = %ld, %s\n", result, strerror(errno));
    return result;
}

int writer(size_t context, uint8_t *data, size_t len) {
    // printf("writer: context = %zu, data = %p, len = %zu\n", context, data, len);
    return fwrite(data, 1, len, (FILE*)context);
}

C2paStream* create_stream(FILE *file) {
    if (file != NULL) {
       return c2pa_create_stream((StreamContext*)file, (ReadCallback)reader, (SeekCallback) seeker, (WriteCallback)writer);
    }
    return NULL;
}

C2paStream* open_file_stream(const char *path, const char* mode) {
    FILE *file = fopen(path, mode);
    if (file != NULL) {
        //printf("file open = %0lx\n", file);
        return c2pa_create_stream((StreamContext*)file, (ReadCallback)reader, (SeekCallback) seeker, (WriteCallback)writer);
    }
    return NULL;
}

int close_file_stream(C2paStream* stream) {
    FILE *file = (FILE*)stream->context;
    int result = fclose(file);
    c2pa_release_stream(stream);
    return result;
}

// Signer callback
intptr_t signer_callback(uint8_t *data, uintptr_t len, uint8_t *signature, uintptr_t sig_max_len) {
    uint64_t data_len= (uint64_t) len;
    //printf("sign: data = %p, len = %ld\n", data, data_len);
    // write data to be signed to a temp file
    int result = save_file("target/c_data.bin", data, data_len);
    if (result < 0) {
        printf("signing failed");
        return -1;
    }
    // sign the temp file by calling openssl in a shell
    system("openssl dgst -sign tests/fixtures/ps256.pem -sha256 -out target/c_signature.sig target/c_data.bin");

    // read the signature file
    FILE* result_file = fopen("target/c_signature.sig", "rb");
    if (result_file == NULL) {
        printf("signing failed");
        return -1;
    }
    fseek(result_file, 0L, SEEK_END);
    long sig_len = ftell(result_file);
    rewind(result_file);

    if (sig_len > sig_max_len) {
        printf("signing failed");
        return -1;
    }
    fread(signature, 1, sig_len, result_file);
    fclose(result_file);
    return sig_len;
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

 
    C2paStream* input_stream = open_file_stream("tests/fixtures/C.jpg", "rb");
    if (input_stream == NULL) {
        printf("error creating input stream = %s\n", c2pa_error());
        return 1;
    }
    printf("input_stream = %0lx\n", (size_t)input_stream);

    ManifestStoreReader* manifest_reader = c2pa_manifest_reader_new();
    if (manifest_reader == NULL) {
        printf("manifest new err = %s\n", c2pa_error());
        return 1;
    }
    char* result = c2pa_manifest_reader_read(&manifest_reader, "image/jpeg", input_stream);
    if (result == NULL) {
        printf("manifest read err = %s\n", c2pa_error());
        return 1;
    }
    close_file_stream(input_stream);
    //c2pa_release_stream(input_stream);

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


	//Open a file to write the thumbnail into
    C2paStream* thumb_stream = open_file_stream("target/thumb_c.jpg", "wb");

    if (thumb_stream == NULL) {
        printf("error creating thumb stream = %s\n", c2pa_error());
        return 1;
    }

    // write the thumbnail resource to the stream
    c2pa_manifest_reader_resource(&manifest_reader, manifest_label, id, thumb_stream);

    free(manifest_label);
    free(id);

    c2pa_release_string(result);
    close_file_stream(thumb_stream);
    // c2pa_release_stream(thumb_stream);
    // fclose(input_file);
    // fclose(thumb_file);


    // create a config
    ManifestBuilderSettingsC settings = { .claim_generator = "python_test"};

    // create a manifest writer
    ManifestBuilder* builder = c2pa_create_manifest_builder(&settings, manifest_json);
    if (builder == NULL) {
        printf("error creating manifest builder = %s\n", c2pa_error());
        return 1;
    }

    // load es256_certs.pem file into a char array
    char * certs = load_file("tests/fixtures/ps256.pub");
    if (certs == NULL) {
        printf("error loading certs = %s\n", c2pa_error());
        return 1;
    }
    // create a sign_info struct
    SignerConfigC config  = { .alg = "ps256", .certs = certs, .time_authority_url = "http://timestamp.digicert.com", .use_ocsp =false };

    // create a signer
    C2paSigner* signer = c2pa_create_signer((SignerCallback)signer_callback, &config);

	//input_stream = create_stream(input_file);
    C2paStream* input_stream2 = open_file_stream("tests/fixtures/A.jpg", "rb");
    if (input_stream2 == NULL) {
        printf("error creating input stream = %s\n", c2pa_error());
        return 1;
    }
    C2paStream* output_stream = open_file_stream("target/c_output.jpg", "wb");
    if (output_stream == NULL) {
        printf("error creating output stream = %s\n", c2pa_error());
        return 1;
    }

    int err = c2pa_manifest_builder_sign(&builder, signer, input_stream2, output_stream);
    if (err != 0) {
        printf("error signing = %s\n", c2pa_error());
        return 1;
    }
    close_file_stream(input_stream2);
    close_file_stream(output_stream);
    c2pa_release_manifest_builder(builder);
    printf("manifest added to: %s\n", "target/c_output.jpg" );
}
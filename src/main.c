// ADOBE CONFIDENTIAL
// Copyright 2022 Adobe
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Adobe and its suppliers, if any. The intellectual
// and technical concepts contained herein are proprietary to Adobe
// and its suppliers and are protected by all applicable intellectual
// property laws, including trade secret and copyright laws.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Adobe.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "c2pa_lib.h"

const char* manifest = "{\
    \"claim_generator\":\"test\",\
    \"ingredients\":[], \
    \"assertions\":[] \
}";

#include <stdio.h>
#include <stdlib.h>

#define SIGN_INFO "{\"alg\" : \"es256\", \"tsa_url\" : \"http://timestamp.digicert.com\", \"signcert\": \"%s\", \"pkey\": \"%s\"}"

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

int main(void) {

    char *version = c2pa_version();
    printf("version = %s\n", version);
    c2pa_release_string(version);

    char *formats = c2pa_supported_formats();
    printf("supported formats = %s\n", formats);
    c2pa_release_string(formats);

    bool has_manifest = c2pa_has_manifest("tests/fixtures/C.jpg");
    printf("has manifest = %d\n", has_manifest);

    char* result = c2pa_verify_from_file("tests/fixtures/C.jpg");
    printf("verify = %s\n", result);
    c2pa_release_string(result);

    result = c2pa_ingredient_from_file("tests/fixtures/C.jpg", "target/tmp", INGREDIENT_THUMBNAIL | INGREDIENT_HASH);
    printf("ingredient = %s\n", result);
    c2pa_release_string(result);

    // // load es256_certs.pem file into a char array
    char * certs = load_file("tests/fixtures/es256_certs.pem");
    // // load es256_private_key.pem file into a char array
    char * private_key = load_file("tests/fixtures/es256_private.key");

    if (certs && private_key) {

        // create a json string with the certs and private key using SIGN_INFO
        char * sign_info = malloc(strlen(SIGN_INFO) + strlen(certs) + strlen(private_key) ); 

        //void *signer = c2pa_create_signer(certs, private_key, "es256", "http://timestamp.digicert.com");
        // printf("sign_info = %s", sign_info);
        if (sign_info) {
            sprintf(sign_info, SIGN_INFO, certs, private_key);  
            result = c2pa_add_manifest_to_file("tests/fixtures/A.jpg", "target/tmp/earth.jpg", manifest, sign_info, false, "http://timestamp.digicert.com");
            // parse result as json and look for "error" key
            if (strstr(result, "\"error\":")) {
                printf("error adding manifest = %s\n", result);
            } else {
                printf("added manifest to %s\n", "target/tmp/earth.jpg");
            }
            free(sign_info);
            c2pa_release_string(result);
            //c2pa_release_signer(signer);
        } else {
            printf("unable to create signer\n");
        }
        free(certs);
        free(private_key);
    } else {
        printf("unable to load certs or private key\n");
    }
}
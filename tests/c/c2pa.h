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
// each license.#include "c2pa.h

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#if defined(_WIN32) || defined(_WIN64)
    #if defined(_STATIC_C2PA) 
        #define IMPORT  __declspec(dllexport)
    #else 
        #if __GNUC__
            #define IMPORT __attribute__ ((dllimport))
        #else
            #define IMPORT __declspec(dllimport)
        #endif
    #endif
#else
    #define IMPORT
#endif

typedef struct StreamContext {

} StreamContext;

typedef intptr_t (*ReadCallback)(const struct StreamContext *context, uint8_t *data, uintptr_t len);

typedef int (*SeekCallback)(const struct StreamContext *context, long offset, int mode);

typedef struct C2paReader {
  struct StreamContext *context;
  ReadCallback read;
  SeekCallback seek;
} C2paReader;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

IMPORT extern
struct C2paReader *c2pa_create_reader(struct StreamContext *context,
                                      ReadCallback read,
                                      SeekCallback seek);

/**
 * Returns a version string for logging
 *
 * # Safety
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 */
IMPORT extern char *c2pa_version(void);

/**
 * Returns a JSON array of supported file format extensions
 *
 * # Safety
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 */
IMPORT extern char *c2pa_supported_extensions(void);

/**
 * Verify a stream and return a ManifestStore report
 *
 * # Errors
 * Returns an error field if there were errors
 *
 * # Safety
 * Reads from a null terminated C string
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 */
IMPORT extern char *c2pa_verify_stream(struct C2paReader reader);

/**
 * Releases a string allocated by Rust
 *
 * # Safety
 * Reads from null terminated C strings
 * The string must not have been modified in C
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_string(char *s);

/**
 * Releases a C2paReader
 *
 * # Safety
 * Reads from null terminated C strings
 * The string must not have been modified in C
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_reader(struct C2paReader *reader);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

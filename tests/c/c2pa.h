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

/**
 * The ManifestStoreReader reads the manifest store from a stream and then
 * provides access to the store via the json() and resource() methods.
 *
 */
typedef struct ManifestStoreReader ManifestStoreReader;

typedef struct StreamContext {

} StreamContext;

typedef intptr_t (*ReadCallback)(const struct StreamContext *context, uint8_t *data, uintptr_t len);

typedef int (*SeekCallback)(const struct StreamContext *context, long offset, int mode);

typedef intptr_t (*WriteCallback)(const struct StreamContext *context,
                                  const uint8_t *data,
                                  uintptr_t len);

typedef struct C2paStream {
  struct StreamContext *context;
  ReadCallback read;
  SeekCallback seek;
  WriteCallback write;
} C2paStream;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Creates a new C2paStream from context with callbacks
 *
 * This allows implementing streams in other languages
 *
 * # Arguments
 * * `context` - a pointer to a StreamContext
 * * `read` - a ReadCallback to read from the stream
 * * `seek` - a SeekCallback to seek in the stream
 * * `write` - a WriteCallback to write to the stream
 *
 * # Safety
 * The context must remain valid for the lifetime of the C2paStream
 * The resulting C2paStream must be released by calling c2pa_release_stream
 *
 */
IMPORT extern
struct C2paStream *c2pa_create_stream(struct StreamContext *context,
                                      ReadCallback read,
                                      SeekCallback seek,
                                      WriteCallback write);

/**
 * Returns the last error message
 *
 * # Safety
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 */
IMPORT extern char *c2pa_error(void);

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
IMPORT extern char *c2pa_verify_stream(struct C2paStream reader);

/**
 * Create a new ManifestStoreReader
 *
 * # Safety
 * The returned value MUST be released by calling release_manifest_reader
 *
 * # Example
 * ```
 * use c2pa::ManifestStoreReader;
 * let reader = ManifestStoreReader::new();
 * ```
 */
IMPORT extern struct ManifestStoreReader *c2pa_manifest_reader_new(void);

/**
 * Read a manifest store from a stream
 *
 * # Arguments
 * * `reader_ptr` - a pointer to a ManifestStoreReader
 * * `format` - the format of the manifest store
 * * `stream` - the stream to read from
 *
 * # Returns
 * * `Result<String>` - the json representation of the manifest store
 *
 * # Example
 * ```
 * use c2pa::ManifestStoreReader;
 * use std::io::Cursor;
 *
 * let reader = ManifestStoreReader::new();
 * let mut stream = Cursor::new("test".as_bytes());
 * let json = reader.read("image/jpeg", &mut stream);
 * ```
 *
 * # Safety
 * Reads from null terminated C strings
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 *
 */
IMPORT extern
char *c2pa_manifest_reader_read(struct ManifestStoreReader **reader_ptr,
                                const char *format,
                                struct C2paStream stream);

/**
 * Writes a resource from the manifest reader to a stream
 *
 * # Arguments
 * * `reader_ptr` - a pointer to a ManifestStoreReader
 * * `manifest_label` - the manifest label
 * * `id` - the resource id
 * * `stream` - the stream to write to
 *
 * # Example
 * ```
 * use c2pa::ManifestStoreReader;
 * use std::io::Cursor;
 *
 * let reader = ManifestStoreReader::new();
 * let mut stream = Cursor::new("test".as_bytes());
 * reader.resource_write("manifest", "id", &mut stream);
 * ```
 *
 * # Safety
 * Reads from null terminated C strings
 *
 * # Errors
 * Returns an error field if there were errors
 *
 */
IMPORT extern
void c2pa_manifest_reader_resource(struct ManifestStoreReader **reader_ptr,
                                   const char *manifest_label,
                                   const char *id,
                                   struct C2paStream stream);

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
 * Releases a C2paStream allocated by Rust
 *
 * # Safety
 * Reads from null terminated C strings
 * The string must not have been modified in C
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_stream(struct C2paStream *stream);

/**
 * Releases a C2paManifestReader allocated by Rust
 *
 * # Safety
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_manifest_reader(struct C2paStream *reader);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

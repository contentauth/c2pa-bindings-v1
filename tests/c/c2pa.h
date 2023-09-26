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

typedef enum SeekMode {
  Start = 0,
  End = 1,
  Current = 2,
} SeekMode;

typedef struct C2paSigner C2paSigner;

typedef struct ManifestBuilder ManifestBuilder;

/**
 * The ManifestStoreReader reads the manifest store from a stream and then
 * provides access to the store via the json() and resource() methods.
 */
typedef struct ManifestStoreReader ManifestStoreReader;

/**
 * Defines a callback to sign data
 */
typedef intptr_t (*SignerCallback)(uint8_t *data,
                                   uintptr_t len,
                                   uint8_t *signature,
                                   intptr_t sig_max_size);

/**
 * Defines the configuration for a Signer
 *
 * # Example
 * ```
 * use c2pa::SignerConfig;
 * let config = SignerConfig {
 *    alg: "Rs256".to_string(),
 *    certs: vec![vec![0; 10]],
 *    time_authority_url: Some("http://example.com".to_string()),
 *    use_ocsp: true,
 * };
 */
typedef struct SignerConfigC {
  /**
   * Returns the algorithm of the Signer.
   */
  const char *alg;
  /**
   * Returns the certificates as a Vec containing a Vec of DER bytes for each certificate.
   */
  const char *certs;
  /**
   * URL for time authority to time stamp the signature
   */
  const char *time_authority_url;
  /**
   * Try to fetch OCSP response for the signing cert if available
   */
  bool use_ocsp;
} SignerConfigC;

/**
 * An Opaque struct to hold a context value for the stream callbacks
 */
typedef struct StreamContext {

} StreamContext;

/**
 * Defines a callback to read from a stream
 */
typedef intptr_t (*ReadCallback)(const struct StreamContext *context, uint8_t *data, uintptr_t len);

/**
 * Defines a callback to seek to an offset in a stream
 */
typedef int (*SeekCallback)(const struct StreamContext *context, long offset, enum SeekMode mode);

/**
 * Defines a callback to write to a stream
 */
typedef intptr_t (*WriteCallback)(const struct StreamContext *context,
                                  const uint8_t *data,
                                  uintptr_t len);

/**
 * A C2paStream is a Rust Read/Write/Seek stream that can be used in C
 */
typedef struct C2paStream {
  struct StreamContext *context;
  ReadCallback read_callback;
  SeekCallback seek_callback;
  WriteCallback write_callback;
} C2paStream;

/**
 * Configuration settings for the ManifestBuilder
 * this is mostly a placeholder for future expansion
 */
typedef struct ManifestBuilderSettingsC {
  const char *claim_generator;
} ManifestBuilderSettingsC;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

IMPORT extern
struct C2paSigner *c2pa_create_signer(SignerCallback signer,
                                      const struct SignerConfigC *config);

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
 * Verify a stream and return a ManifestStore report
 *
 * # Errors
 * Returns an error field if there were errors
 *
 * # Safety
 * The returned value MUST be released by calling release_string
 * and it is no longer valid after that call.
 */
IMPORT extern char *c2pa_verify_stream(struct C2paStream *reader);

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
                                struct C2paStream *stream);

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
                                   struct C2paStream *stream);

/**
 * Create a ManifestBuilder
 *
 * # Arguments
 * * `settings` - a pointer to a ManifestBuilderSettingsC
 * * `json` - a pointer to a null terminated JSON Manifest Definition
 *
 * # Returns
 * * `Result<*mut ManifestBuilder>` - a pointer to a ManifestBuilder
 *
 * # Safety
 * The returned value MUST be released by calling release_manifest_builder
 *
 * # Example
 * ```
 * use c2pa::{ManifestBuilder, ManifestBuilderSettings};
 * let json = r#"{
 *     "claim_generator": "test_generator",
 *     "format": "image/jpeg",
 *     "title": "test_title"
 * }"#;
 * let settings = ManifestBuilderSettings {
 *    generator: "test".to_string(),
 * };
 *
 *   let builder = ManifestBuilder::new(&settings);
 *    builder.from_json(json);
 * ```
 *
 */
IMPORT extern
struct ManifestBuilder *c2pa_create_manifest_builder(const struct ManifestBuilderSettingsC *settings,
                                                     const char *json);

/**
 * Sign using a ManifestBuilder
 *
 * # Arguments
 * * `builder` - a pointer to a ManifestBuilder
 * * `signer` - a pointer to a C2paSigner
 * * `input` - a pointer to a C2paStream
 * * `output` - optional pointer to a C2paStream
 *
 */
IMPORT extern
int c2pa_manifest_builder_sign(struct ManifestBuilder **builder_ptr,
                               const struct C2paSigner *signer,
                               struct C2paStream *input,
                               struct C2paStream *output);

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
 * Releases a ManifestStoreReader allocated by Rust
 *
 * # Safety
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_manifest_reader(struct ManifestStoreReader *reader);

/**
 * Releases a ManifestBuilder allocated by Rust
 *
 * # Safety
 * can only be released once and is invalid after this call
 */
IMPORT extern void c2pa_release_manifest_builder(struct ManifestBuilder *builder);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

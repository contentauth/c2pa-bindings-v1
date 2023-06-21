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

/** @file adobe_c2pa.h
 *  @brief High level C API for the c2pa_rs crate
 */
#include <stdint.h>
#include <stdbool.h>

#if defined(_WIN32) || defined(_WIN64)
    #if defined(_STATIC_C2PA) 
        #define IMPORT
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

#ifdef __cplusplus
extern "C"
{
#endif

// Embed options for adobe_c2pa_ingredient_from_file
#define INGREDIENT_DEFAULT     0x00   // no options 
#define INGREDIENT_THUMBNAIL   0x01   // generate a thumbnail if needed
#define INGREDIENT_HASH        0x02   // add a blake3 asset hash

// Embed options for adobe_c2pa_add_manifest (One option must be set)
#define DEST_OPTION_EMBED   0x01   // embeds the manifest in the asset 
#define DEST_OPTION_SIDECAR 0x02   // writes the manifest to a .c2pa file
#define DEST_OPTION_CLOUD   0x84   // Upload and embed URL in XMP
#define DEST_OPTION_BOTH    0x85   // Upload, embed manifest in file and add URL to XMP

/** @brief Provides the information needed to create a signer
 * 
 * signcert: pointer a PEM encoded certificate 
 * pkey: pointer to a PEM encoded private key
 * alg: signing algorithm ( es256, es384, es512, ps256, ps384, ps512, ed25519)
 * tsa_url: optional URL to a time stamping authority
 */
typedef struct {
    const char* signcert;
    const char* pkey;
    const char* alg;
    const char* tsa_url;
} SignInfo;

/** @brief returns a version string for this library
 *
 *  This is formatted as a UserAgent with space separated
 *  agent/version values, similar to the claim_generator field.
 *  i.e: "adobe_c2pa/0.1.0 c2pa-rs/0.14.1"
 *
 *  @return char* to a version string (must be released with release_string).
 */
IMPORT extern char* c2pa_version();

/** @brief returns JSON array of supported filetypes as three-letter file extensions
*/
extern char* c2pa_supported_formats();

/** @brief returns true (1) if the file appears to have a manifest (without validating) else false (0)
*/
extern bool c2pa_has_manifest(const char* path); 


/** @brief Validates a file and returns a ManifestStore report
 *
 *  This will perform a full c2pa validation on the file.
 * 
 *  Success: the response will include a manifest_store field.
 *  Fail: the response will include an error field.
 *  Warning: Response can be NULL in rare cases
 * 
 *  A ManifestStore can contain many manifests linked together
 *  as ingredients of each other. The active manifest is the most
 *  recently added. All other manifests are referenced as ingredients.
 *
 *  @param path Path to a file to verify.
 *  @return char* to a response structure in JSON (must be released with release_string).
 */
IMPORT extern char* c2pa_verify_from_file(const char *path);

/** @brief Creates a C2PA Ingredient from a file
 *
 *  Success: the response will include an ingredient field.
 *  Fail: the response will include an error field.
 *  Warning: Response can be NULL in rare cases
 * 
 *  Ingredients can be generated from files with or without manifests
 *  If a file validates fully, this will extract an existing thumbnail
 *  otherwise it will try to generate one for supported file types if
 *  make_thumbnails is true. If the file has a manifest store, it wll be 
 *  extracted as a .c2pa file in the data_dir folder. 
 * 
 *  A returned ingredient stucture contains the data necessary to create
 *  and ingredient with the add_manifest call. 
 * 
 *  If the asset has been tampered with since the active manifest was added, the 
 *  Ingredient will contain a validation_status field reporting any issues.
 *  This validation_status must be included when adding ingredients.
 *
 *  data_dir folders will be created if needed 
 * 
 *  @param path Path to a file to convert to an ingredient.
 *  @param data_dir Path to a folder to hold ingredient thumbnail and manifest file.
 *  @param flags uint8_t - see INGREDIENT_ flags above
 *  @return char* to a response structure in JSON (must be released with release_string).
 */
IMPORT extern char* c2pa_ingredient_from_file(const char *path, const char* data_dir, uint8_t flags);

IMPORT extern void* c2pa_create_signer(const char *signcert, const char* pkey, const char* alg, const char*tsa_url);

/** @brief Adds a C2PA Manifest to a file
 * 
 *  Success: the response will include a url field if cloud was set to true
 *  Fail: the response will include an error field.
 *  Warning: Response can be NULL in rare cases
 * 
 *  Warning; the destination file will be overwritten if it already exists
 *  The destination file type must be the same as the source, but the name can be different
 * 
 *  Ingredients can be created using ingredient_from_file.
 *  If there is a parent ingredient, make sure to set is_parent to true.
 *  Validation_status fields must be preserved when adding ingredients.
 * 
 *  Manifests should always define at least one assertion.
 *  
 *  @param source_path Path to a file to which we want to add a manifest
 *  @param dest_path Path to write a file with the manifest added. (can be the same as source_path to overwrite)
 *  @param manifest string containing a Manifest Json definition of the manifest to create
 *  @param sign_info a SignInfo structure defining the signature parameters
 *  @param dest_option Destination settings - See DEST_OPTION_ flags above 
 *  @return char* to a response structure in JSON (must be released with release_string)
 */
IMPORT extern char* c2pa_add_manifest_to_file(const char *source_path, const char *dest_path, const char* manifest, SignInfo signer, bool side_car, const char* remote_url);


/** @brief Release string values returned by these functions
 * 
 *  The return values from the above functions are allocated in Rust
 *  and must be released with this fuction when no longer needed. 
 * 
 *  The value is invalid after this call and should never be referenced.
 * 
 *  @param s char* returned from any of the above functions
 */
IMPORT extern void c2pa_release_string(char *s);

/** @brief Release signer returned from c2pa_create_signer
 * 
 *  The value is invalid after this call and should never be referenced.
 * 
 *  @param signer void* returned c2pa_create_signer
 */
IMPORT extern void c2pa_release_signer(void *signer);

#ifdef __cplusplus
}
#endif

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

/// This module exports a cai_helper library
use c2pa::{Ingredient, ManifestStore};
//use tokio::runtime::Runtime;

//mod c_api;
mod response;
pub use response::{ErrorResponse, Response};
//mod remote_signer;
//pub(crate) use remote_signer::RemoteSigner;

uniffi::include_scaffolding!("c2pa_uniffi");

#[derive(Debug, thiserror::Error)]
pub enum C2paError {
    #[error("Error converting manifest: {0} - {1}")]
    AssertionConversion(String, String),

    #[error("Error converting manifest: {0}")]
    ManifestConversion(String),

    #[error("Could not read input path: {0}")]
    InputReadError(String),

    #[error("Invalid signing algorithm was supplied: {0}")]
    InvalidAlgorithm(String),

    #[error(transparent)]
    Sdk(#[from] c2pa::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Pass in a file path and return a Manifest Store Result string in JSON format
pub fn verify_from_file(path: &str) -> Result<String, C2paError> {
    Ok(ManifestStore::from_file(path)
        .map_err(C2paError::Sdk)?
        .to_string())
}

/// Pass in a file path and return a Manifest Store Result string in JSON format
pub fn c2pa_verify_from_file(path: &str) -> String {
    Response::from_result(ManifestStore::from_file(path)).to_string()
}

/// Pass in a file path and return Result with Ingredient.
///
/// Thumbnail and c2pa data written to data_path
/// make_thumbnail true to generate thumbnails when possible
pub fn c2pa_ingredient_from_file(path: &str, data_dir: &str) -> String {
    Response::from_result(Ingredient::from_file_with_folder(path, data_dir)).to_string()
}

// Create a signed manifest for the file at source_path, writing to dest_path.
//
// using the passed manifest and Adobe auth token
// upload manifest to cloud if cloud is true, otherwise embed in file
// pub fn adobe_add_manifest(
//     source_path: &str,
//     dest_path: &str,
//     manifest: &str,
//     auth_token: &str,
//     dest_option: DestOption,
// ) -> String {
//     match ManifestJson::from_bytes(manifest.as_bytes()) {
//         Ok(manifest_json) => {
//             let mut rt = Runtime::new().unwrap();
//             rt.block_on(async {
//                 match manifest_json
//                     .add_to_file(&source_path, &dest_path, auth_token, dest_option)
//                     .await
//                 {
//                     Ok(url) => Response::from_url(url),
//                     Err(error) => Response::from_error(ErrorResponse::new(error)),
//                 }
//             })
//         }
//         Err(error) => Response::from_error(ErrorResponse::new(error)),
//     }
//     .to_string()
// }

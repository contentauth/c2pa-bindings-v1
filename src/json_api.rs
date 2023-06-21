use c2pa::{Error, Ingredient, Manifest, ManifestStore};
use std::path::PathBuf;

// pub struct SignerData {
//     signer: std::sync::Arc<Box<dyn Signer>>,
//     tsa_url: Option<String>,
// }
use crate::SignerInfo;

// pub struct C2paSigner {
//     signer: Option<Box<dyn Signer>>
// }

// impl C2paSigner {
//     pub fn new() -> Self {
//         Self { signer: None }
//     }

//     pub fn from_keys(signcert: &str, pkey: &str, alg: &str, tsa_url: Option<&str>) -> C2paSigner {
//         let alg = alg
//         .to_lowercase()
//         .parse()
//         .map_err(|_| c2pa::Error::UnsupportedType).unwrap;
//         let tsa_url = tsa_url.map(|s| s.to_string());
//         let signer = create_signer::from_keys(signcert.as_bytes(), pkey.as_bytes(), alg, tsa_url).unwrap();
//         Self {
//             signer: Some(signer)
//         }
//     }

//     fn signer(&self) -> Result<&dyn Signer, Error> {
//         self.signer.as_ref().ok_or(Error::NotFound).map(|s| s.as_ref())
//     }
// }

/// Pass in a file path and return a ManifestStore Result string in JSON format
///
pub fn verify_from_file_json(path: &str) -> Result<String, Error> {
    Ok(ManifestStore::from_file(path)?.to_string())
}

/// Pass in a file path and return Result with a JSON Ingredient.
///
/// Thumbnail and c2pa data written to data_path
/// make_thumbnail true to generate thumbnails when possible
pub fn ingredient_from_file_json(path: &str, data_dir: &str) -> Result<String, Error> {
    Ok(Ingredient::from_file_with_folder(path, data_dir)?.to_string())
}

// pub fn create_signer(signcert: &str, pkey: &str, alg: &str, tsa_url: Option<&str>) -> Result<Box<dyn Signer>, Error> {
//     let alg = alg
//     .to_lowercase()
//     .parse()
//     .map_err(|_| c2pa::Error::UnsupportedType)?;
//     let tsa_url = tsa_url.map(|s| s.to_string());
//     create_signer::from_keys(signcert.as_bytes(), pkey.as_bytes(), alg, tsa_url)
// }

/// Pass in a source and destination file path, a Manifest, and SignerInfo in JSON
/// and return Result with the binary manifest generated.
///
/// Any file paths in the manifest will be read relative to the source file
pub fn add_manifest_to_file_json(
    source: &str,
    dest: &str,
    manifest_info: &str,
    signer_info: SignerInfo,
    side_car: bool,
    remote_url: Option<String>,
) -> Result<Vec<u8>, Error> {

    let mut manifest = Manifest::from_json(manifest_info)?;

    // read any manifest referenced files from the source path
    // or current folder if no path available
    if let Some(path) = PathBuf::from(source).parent() {
        manifest.with_base_path(path)?;
    } else if let Ok(path) = std::env::current_dir() {
        manifest.with_base_path(&path)?;
    }

    // if side_car then don't embed the manifest
    if side_car {
        manifest.set_sidecar_manifest();
    }

    // add the remote url if provided
    if let Some(url) = remote_url {
        if side_car {
            manifest.set_remote_manifest(url);
        } else {
            manifest.set_embedded_manifest_with_remote_ref(url);
        }
    }

    // If the source file has a manifest store, and no parent is specified, treat the source's manifest store as the parent.
    if manifest.parent().is_none() {
        let source_ingredient = Ingredient::from_file(source)?;
        if source_ingredient.manifest_data().is_some() {
            manifest.set_parent(source_ingredient)?;
        }
    }

    manifest.embed(&source, &dest, signer_info.signer()?.as_ref())
}

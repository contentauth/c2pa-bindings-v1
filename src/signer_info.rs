// ADOBE CONFIDENTIAL
// Copyright 2023 Adobe
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

use c2pa::{create_signer, Error, Result, Signer, SigningAlg};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct SignerInfo {
    pub signcert: Vec<u8>,
    pub pkey: Vec<u8>,
    pub alg: String,
    pub tsa_url: Option<String>,
}
impl SignerInfo {
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Error::JsonError)
    }

    pub fn alg(&self) -> Result<SigningAlg> {
        self.alg
            .to_lowercase()
            .parse()
            .map_err(|_| c2pa::Error::UnsupportedType)
    }

    pub fn signer(&self) -> Result<Box<dyn Signer>> {
        create_signer::from_keys(
            &self.signcert,
            &self.pkey,
            self.alg()?,
            self.tsa_url.clone(),
        )
    }
}

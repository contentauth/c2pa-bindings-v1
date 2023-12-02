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
// each license.

use c2pa::Error::OpenSslError;
use openssl::{error::ErrorStack, hash::MessageDigest, pkey::PKey, rsa::Rsa};

use crate::signer::{SignerCallback, SignerConfig};
use crate::stream::{StreamError, StreamResult};
use crate::{C2paError, Result};

pub(crate) struct TestSigner {
    private_key: Vec<u8>,
}

impl TestSigner {
    pub fn new() -> Self {
        Self {
            private_key: include_bytes!("../tests/fixtures/ps256.pem").to_vec(),
        }
    }

    pub fn config(&self) -> SignerConfig {
        SignerConfig {
            alg: "ps256".to_string(),
            certs: include_bytes!("../tests/fixtures/ps256.pub").to_vec(),
            time_authority_url: None,
            use_ocsp: false,
        }
    }
}

impl SignerCallback for TestSigner {
    fn sign(&self, data: Vec<u8>) -> StreamResult<Vec<u8>> {
        local_sign(&data, &self.private_key).map_err(|e| StreamError::Other {
            reason: e.to_string(),
        })
    }
}

pub fn local_sign(data: &[u8], pkey: &[u8]) -> Result<Vec<u8>> {
    openssl_rsa256_sign(data, pkey).map_err(|e| C2paError::from(OpenSslError(e)))
}

fn openssl_rsa256_sign(data: &[u8], pkey: &[u8]) -> std::result::Result<Vec<u8>, ErrorStack> {
    let rsa = Rsa::private_key_from_pem(pkey)?;
    let pkey = PKey::from_rsa(rsa)?;
    let mut signer = openssl::sign::Signer::new(MessageDigest::sha256(), &pkey)?;

    signer.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?; // use C2PA recommended padding
    signer.set_rsa_mgf1_md(MessageDigest::sha256())?;
    signer.set_rsa_pss_saltlen(openssl::sign::RsaPssSaltlen::DIGEST_LENGTH)?;

    signer.sign_oneshot_to_vec(data)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::test_stream::TestStream;

//     #[test]
//     fn test_sign() {
//     }
// }

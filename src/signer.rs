use std::str::FromStr;
use std::sync::RwLock;

use crate::error::{C2paError, Result};
use crate::stream::StreamResult;

#[repr(C)]
pub struct SignerConfig {
    /// Returns the algorithm of the Signer.
    pub alg: String,

    /// Returns the certificates as a Vec containing a Vec of DER bytes for each certificate.
    pub certs: Vec<u8>,

    /// URL for time authority to time stamp the signature
    pub time_authority_url: Option<String>,

    /// Try to fetch OCSP response for the signing cert if available
    pub use_ocsp: bool,
}

struct SignerInternalConfig {
    /// The algorithm of the Signer as Signing Alg
    alg: c2pa::SigningAlg,

    /// The certificates as a Vec containing a Vec of DER bytes for each certificate.
    certs: Vec<Vec<u8>>,

    /// The size in bytes of the largest possible expected signature.
    /// Signing will fail if the result of the `sign` function is larger
    /// than this value.
    reserve_size: u64,

    /// URL for time authority to time stamp the signature
    time_authority_url: Option<String>,

    /// OCSP response for the signing cert if available
    ocsp_val: Option<Vec<u8>>,
}

pub struct C2paSigner {
    callback: Box<dyn SignerCallback>,

    settings: RwLock<SignerInternalConfig>,
}

impl C2paSigner {
    pub fn new(callback: Box<dyn SignerCallback>) -> Self {
        Self {
            callback,
            settings: RwLock::new(SignerInternalConfig {
                alg: c2pa::SigningAlg::Ps256,
                certs: Vec::new(),
                reserve_size: 1024,
                time_authority_url: None,
                ocsp_val: None,
            }),
        }
    }

    // fn certs(pem_certs: &[u8]) -> Result<Vec<Vec<u8>>> {
    //     let signcerts = openssl::x509::X509::stack_from_pem(pem_certs)
    //         .map_err(|e| C2paError::Sdk(c2pa::Error::OpenSslError(e)))?;
    //     let mut certs: Vec<Vec<u8>> = Vec::new();
    //     for c in signcerts {
    //         let cert = c
    //             .to_der()
    //             .map_err(|e| C2paError::Sdk(c2pa::Error::OpenSslError(e)))?;
    //         certs.push(cert);
    //     }
    //     Ok(certs)
    // }

    pub fn configure(&self, config: &SignerConfig) -> Result<()> {
        if let Ok(mut settings) = RwLock::write(&self.settings) {
            settings.alg = c2pa::SigningAlg::from_str(&config.alg)
                .map_err(|e| C2paError::Other(e.to_string()))?;
            let mut pems =
                pem::parse_many(&config.certs).map_err(|e| C2paError::Other(e.to_string()))?;
            settings.certs = pems.drain(..).map(|p| p.into_contents()).collect();

            //settings.certs = Self::certs(&config.certs).expect("Failed to parse certs"  );
            //println!("certs = {:?}", settings.certs);
            settings.reserve_size = config.certs.len() as u64 + 20000; /* todo: call out to TSA to get actual timestamp and use that size */

            settings.time_authority_url = config.time_authority_url.clone();
            settings.ocsp_val = None;
        } else {
            return Err(C2paError::Other("RwLock".to_string()));
        }
        Ok(())
    }
}

impl c2pa::Signer for C2paSigner {
    fn sign(&self, data: &[u8]) -> c2pa::Result<Vec<u8>> {
        self.callback
            .sign(data.to_vec())
            .map_err(|e| c2pa::Error::BadParam(e.to_string()))
    }

    fn alg(&self) -> c2pa::SigningAlg {
        RwLock::read(&self.settings).unwrap().alg
    }

    fn certs(&self) -> c2pa::Result<Vec<Vec<u8>>> {
        Ok(RwLock::read(&self.settings).unwrap().certs.clone())
    }

    fn reserve_size(&self) -> usize {
        RwLock::read(&self.settings).unwrap().reserve_size as usize
    }

    fn time_authority_url(&self) -> Option<String> {
        RwLock::read(&self.settings)
            .unwrap()
            .time_authority_url
            .clone()
    }

    fn ocsp_val(&self) -> Option<Vec<u8>> {
        RwLock::read(&self.settings).unwrap().ocsp_val.clone()
    }
}

pub trait SignerCallback: Send + Sync {
    /// Read a stream of bytes from the stream
    fn sign(&self, bytes: Vec<u8>) -> StreamResult<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_signer::TestSigner;
    use c2pa::Signer;

    #[test]
    fn test_sign() {
        let signer = Box::new(TestSigner::new());
        let config = signer.config();
        let data = b"some sample content to sign";

        let signer = C2paSigner::new(signer);
        signer.configure(&config).expect("Signer config failed");

        let signature = signer.sign(data).unwrap();
        println!("signature len = {}", signature.len());
        assert!(signature.len() <= signer.reserve_size());
    }
}

// Copyright 2022 Adobe. All rights reserved.
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

use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum StreamError {
//     #[error("IoError")]
//     IoError,
//     #[error("Other Error: {reason}")]
//     Other { reason: String },
//     #[error("InternalStreamError")]
//     InternalStreamError,
// }

// impl From<uniffi::UnexpectedUniFFICallbackError> for StreamError {
//     fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
//         let err = Self::Other {
//             reason: err.reason.clone(),
//         };
//         println!("{:#}", err);
//         err
//     }
// }

// impl From<C2paError> for StreamError {
//     fn from(e: C2paError) -> Self {
//         Self::Other {
//             reason: e.to_string(),
//         }
//     }
// }

use crate::StreamError;

#[derive(Error, Debug)]
pub enum C2paError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sdk(#[from] c2pa::Error),

    #[error(transparent)]
    Stream(StreamError),

    #[error("Read Write Lock failure")]
    RwLock,
}

pub type Result<T> = std::result::Result<T, C2paError>;

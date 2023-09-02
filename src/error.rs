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
use std::cell::RefCell;

use thiserror::Error;

// LAST_ERROR handling borrowed Copyright (c) 2018 Michael Bryan
thread_local! {
    static LAST_ERROR: RefCell<Option<C2paError>> = RefCell::new(None);
}

// /// Take the most recent error, clearing `LAST_ERROR` in the process.
// pub fn take_last_error() -> Option<C2paError> {
//     LAST_ERROR.with(|prev| prev.borrow_mut().take())
// }

// /// Update the `thread_local` error, taking ownership of the `Error`.
// pub fn update_last_error<E: Into<C2paError>>(err: E) {
//     LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(err.into()));
// }

// /// Peek at the most recent error and get its error message as a Rust `String`.
// pub fn error_message() -> Option<String> {
//     LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| format!("{:#}", e)))
// }

use crate::StreamError;

#[derive(Error, Debug)]
pub enum C2paError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sdk(#[from] c2pa::Error),

    #[error("Api Error: {0}")]
    Ffi(String),

    #[error(transparent)]
    Stream(StreamError),

    #[error("Read Write Lock failure")]
    RwLock,
}

impl C2paError {
    /// Returns the last error as String
    pub fn last_message() -> Option<String> {
        LAST_ERROR.with(|prev| prev.borrow().as_ref().map(|e| e.to_string()))
    }

    /// Sets the last error
    pub fn set_last(self) {
        LAST_ERROR.with(|prev| *prev.borrow_mut() = Some(self));
    }

    /// Takes the the last error and clears it
    pub fn take_last() -> Option<C2paError> {
        LAST_ERROR.with(|prev| prev.borrow_mut().take())
    }
}

impl From<StreamError> for C2paError {
    fn from(e: StreamError) -> Self {
        match e {
            StreamError::Io{ reason }  => Self::Io(std::io::Error::new(std::io::ErrorKind::Other, reason)),
            StreamError::Other { reason } => Self::Ffi(reason),
            StreamError::InternalStreamError => Self::Stream(e),
        }
    }
}


pub type Result<T> = std::result::Result<T, C2paError>;

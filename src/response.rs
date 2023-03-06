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

//use log::warn;
use c2pa::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// JSON serializable struct containing success and error responses
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    /// If there was an error, this contains the `ErrorResponse`
    Error(ErrorResponse),
    /// Success, with serialized JSON result
    Ok(Value),
}

impl Response {
    pub fn from_result<T>(result: Result<T>) -> Self
    where
        T: Serialize,
    {
        match result {
            Ok(value) => match serde_json::to_value(value) {
                Ok(value) => Self::Ok(value),
                Err(e) => Self::Error(ErrorResponse::new(e)),
            },
            Err(error) => Self::Error(ErrorResponse::new(error)),
        }
    }

    // Returns an error response
    pub fn from_error(error: ErrorResponse) -> Self {
        Self::Error(error)
    }
}

/// This helps categorize errors for upstream error handling
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCode {
    /// A referenced file was not found
    NotFound,
    /// Operation could not be completed due to being offline
    Offline,
    /// All other errors
    Other,
    /// File or folder access permission failure
    Permission,
    /// The user_token is not authorized for this operation
    Unauthorized,
}

/// JSON serializable error with message, code and context
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// A message describing the error
    pub message: String,
    /// An ErrorCode to help interpret the error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    /// Additional context for the cause of the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let report = serde_json::to_string_pretty(self).unwrap_or_default();
        f.write_str(&report)
    }
}

impl ErrorResponse {
    /// This tries to generate ErrorCode values for from the text of certain errors
    /// The caller can use the code to perform special error handling in these cases
    fn find_code(error: &str) -> Option<ErrorCode> {
        // todo: these are macosx specific, need to add checks for similar errors on Windows and linux
        Some(match error {
            err if err.contains("(os error 2)") => ErrorCode::NotFound,
            err if err.contains("Permission") || err.contains("(os error 1)") => {
                ErrorCode::Permission
            }
            err if err.contains("dns error") => ErrorCode::Offline,
            _ => ErrorCode::Other,
        })
    }

    pub fn new<V: std::fmt::Display>(error: V) -> Self {
        let error = format!("{}", error);
        let code = Self::find_code(&error);
        //warn!("{}", &error);
        Self {
            message: error,
            code,
            context: None,
        }
    }

    pub fn set_code(mut self, code: ErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    pub fn set_context<S: Into<String>>(mut self, context: S) -> Self {
        self.context = Some(context.into());
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_code() {
        let resp = ErrorResponse::new("Some other error").set_context("foo");
        assert_eq!(resp.code, Some(ErrorCode::Other));
        assert_eq!(resp.context, Some("foo".to_string()));
    }

    #[test]
    fn test_set_code() {
        let resp = ErrorResponse::new("Some other error").set_code(ErrorCode::Offline);
        assert_eq!(resp.code, Some(ErrorCode::Offline));
        assert_eq!(resp.context, None);
    }

    #[test]
    fn test_permission() {
        let resp = ErrorResponse::new("A Permission error").set_context("foo");
        assert_eq!(resp.code, Some(ErrorCode::Permission));
        assert_eq!(resp.context, Some("foo".to_string()));
    }

    #[test]
    fn test_not_found() {
        let resp = ErrorResponse::new("(os error 2)");
        assert_eq!(resp.code, Some(ErrorCode::NotFound));
        assert_eq!(resp.context, None);
    }

    #[test]
    fn test_from_error() {
        let resp = Response::from_error(ErrorResponse::new("(os error 2)"));
        println!("{}", resp);
        if let Response::Error(err) = resp {
            assert_eq!(err.code, Some(ErrorCode::NotFound))
        } else {
            panic!("wrong response")
        }
    }

    #[test]
    fn test_from_url() {
        let url = "http:://foo.org".to_string();
        let resp = Response::from_result(Ok(url.clone()));
        if let Response::Ok(url2) = resp {
            assert_eq!(url2, Some(url))
        } else {
            panic!("wrong response")
        }
    }
}

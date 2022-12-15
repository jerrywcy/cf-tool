#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use std::time::SystemTime;

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

use super::error::NoAuthorizationError;

fn rand() -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    let len = 6;
    let mut rng = rand::thread_rng();

    (0..len)
        .map(|_| charset[rng.gen_range(0..charset.len())] as char)
        .collect::<String>()
}

fn hash(s: String) -> String {
    let mut hasher = Sha512::new();
    hasher.update(s);
    format!("{:x}", hasher.finalize())
}

pub(super) struct CFApiUrl {
    baseurl: String,
    endpoint: String,
    parameters: Vec<(String, String)>,
}

pub static BASEURL: &str = "https://codeforces.com/";
pub static API_BASEURL: &str = "https://codeforces.com/api/";

impl CFApiUrl {
    pub fn new(endpoint: &str) -> Self {
        Self {
            baseurl: String::from(API_BASEURL),
            endpoint: String::from(endpoint),
            parameters: vec![],
        }
    }

    pub fn add_parameter(&mut self, key: impl ToString, value: Option<impl ToString>) -> &mut Self {
        if let Some(value) = value {
            self.parameters.push((key.to_string(), value.to_string()));
        }
        self
    }

    pub fn add_required_parameter(
        &mut self,
        key: impl ToString,
        value: impl ToString,
    ) -> &mut Self {
        self.parameters.push((key.to_string(), value.to_string()));
        self
    }

    pub fn authorize(&mut self, key: &str, secret: &str) -> String {
        self.add_parameter("apiKey", Some(key));
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.add_parameter("time", Some(time));
        let mut parameters = self.parameters.clone();
        parameters.sort();
        let parameters = parameters
            .iter()
            .enumerate()
            .map(|(index, (key, value))| {
                if index == 0 {
                    format!("?{key}={value}")
                } else {
                    format!("&{key}={value}")
                }
            })
            .collect::<String>();
        let rand = rand();
        let hash = hash(format!(
            "{}/{}{}#{}",
            rand, self.endpoint, parameters, secret
        ));
        let apiSig = format!("{rand}{hash}");
        self.add_parameter("apiSig", Some(apiSig));
        self.into_url()
    }

    pub fn into_url(&mut self) -> String {
        format!(
            "{}{}{}",
            self.baseurl,
            self.endpoint,
            self.parameters
                .iter()
                .enumerate()
                .map(|(index, (key, value))| {
                    if index == 0 {
                        format!("?{key}={value}")
                    } else {
                        format!("&{key}={value}")
                    }
                })
                .collect::<String>()
        )
    }
}

/// Represent the status of a given response
#[derive(Debug, Deserialize, Serialize)]
pub enum CFApiResponseStatus {
    OK,
    FAILED,
}

/// Represent response by CodeForces API
#[derive(Debug, Serialize, Deserialize)]
pub struct CFApiResponse<T> {
    /// either "OK" or "FAILED"
    pub status: CFApiResponseStatus,
    /// only available when [`status`] is "OK"
    pub result: Option<T>,
    /// only available when [`status`] is "FAILED"
    pub comment: Option<String>,
}

pub(super) fn get_authorize<'a>(
    key: &'a Option<String>,
    secret: &'a Option<String>,
) -> Result<(&'a str, &'a str), NoAuthorizationError> {
    match (key, secret) {
        (None, None) => Err(NoAuthorizationError::new("key and secret")),
        (None, Some(_)) => Err(NoAuthorizationError::new("key")),
        (Some(_), None) => Err(NoAuthorizationError::new("secret")),
        (Some(key), Some(secret)) => Ok((key, secret)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rand_test {
        use super::rand;
        #[test]
        fn rand_len() {
            for _ in 0..1000 {
                let result = rand();
                assert_eq!(
                    result.len(),
                    6,
                    "Len of string {result} generated by rand() isn't 6."
                );
            }
        }

        #[test]
        fn rand_charset() {
            let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";
            for _ in 0..1000 {
                let result = rand();
                for c in result.as_bytes() {
                    assert!(
                    charset.contains(c),
                    "String {result} generated by rand() contains characters not in given charset."
                );
                }
            }
        }
    }

    mod hash_test {
        use super::hash;
        #[test]
        fn hash_test() {
            assert_eq!(hash("test".to_string()),"ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff");
        }
    }

    mod CFApiUrl_test {
        use super::CFApiUrl;
        #[test]
        fn url_endpoint() {
            let url = CFApiUrl::new("endpoint").into_url();
            assert_eq!(url, "https://codeforces.com/api/endpoint");
        }

        #[test]
        fn url_parameter_with_value() {
            let url = CFApiUrl::new("endpoint")
                .add_parameter("parameter", Some("value"))
                .into_url();
            assert_eq!(url, "https://codeforces.com/api/endpoint?parameter=value");
        }

        #[test]
        fn url_parameter_without_value() {
            let url = CFApiUrl::new("endpoint")
                .add_parameter("parameter", None::<String>)
                .into_url();
            assert_eq!(url, "https://codeforces.com/api/endpoint");
        }

        #[test]
        fn url_required_parameter() {
            let url = CFApiUrl::new("endpoint")
                .add_required_parameter("parameter", "value")
                .into_url();
            assert_eq!(url, "https://codeforces.com/api/endpoint?parameter=value");
        }

        #[test]
        fn url_general() {
            let url = CFApiUrl::new("endpoint")
                .add_required_parameter("required_parameter", "required_value")
                .add_parameter("parameter", Some("value"))
                .add_parameter("doesnt_exist", None::<String>)
                .into_url();
            assert_eq!(url, "https://codeforces.com/api/endpoint?required_parameter=required_value&parameter=value");
        }
    }

    mod get_authorize_test {
        use super::get_authorize;
        use crate::api::error::NoAuthorizationError;

        #[test]
        fn get_authorize_with_key_and_secret_missing() {
            let key = None::<String>;
            let secret = None::<String>;
            match get_authorize(&key, &secret) {
                Err(err) => assert_eq!(
                    err,
                    NoAuthorizationError::new("key and secret"),
                    "get_authorize returned wrong error message"
                ),
                Ok(_) => panic!("get_authorize returned OK with key and secret missing"),
            }
        }

        #[test]
        fn get_authorize_with_key_missing() {
            let key = None::<String>;
            let secret = Some("secret".to_string());
            match get_authorize(&key, &secret) {
                Err(err) => assert_eq!(
                    err,
                    NoAuthorizationError::new("key"),
                    "get_authorize returned wrong error message"
                ),
                Ok(_) => panic!("get_authorize returned OK with key missing"),
            }
        }

        #[test]
        fn get_authorize_with_secret_missing() {
            let key = Some("key".to_string());
            let secret = None::<String>;
            match get_authorize(&key, &secret) {
                Err(err) => assert_eq!(
                    err,
                    NoAuthorizationError::new("secret"),
                    "get_authorize returned wrong error message"
                ),
                Ok(_) => panic!("get_authorize returned OK with secret missing"),
            }
        }

        #[test]
        fn get_authorize_with_key_and_secret_configured() {
            let key = Some("key".to_string());
            let secret = Some("secret".to_string());
            match get_authorize(&key, &secret) {
                Ok((key, secret)) => {
                    assert_eq!(key, "key");
                    assert_eq!(secret, "secret");
                }
                Err(_) => panic!("get_authorize returned Error with key and secret configured"),
            }
        }
    }
}

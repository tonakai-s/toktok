use std::str::FromStr;

use anyhow::bail;
use reqwest::{
    Client, RequestBuilder, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use yaml_rust2::Yaml;

use crate::{
    checker::structs::{CheckerResult, CheckerStatus},
    configuration::ConfigurationParseError,
};

#[derive(Debug)]
pub struct WebChecker {
    req_builder: RequestBuilder,
    expected_code: StatusCode,
}

impl WebChecker {
    pub fn new(
        domain: String,
        path: Option<String>,
        expected_code: StatusCode,
        headers: Option<HeaderMap>,
    ) -> Self {
        let url = path.as_ref().map(|path| format!("{domain}{path}"));

        let client = Client::new()
            .get(url.unwrap_or(domain))
            .headers(headers.unwrap_or_default());

        Self {
            req_builder: client,
            expected_code,
        }
    }

    pub fn expected_code(&self) -> &StatusCode {
        &self.expected_code
    }

    pub async fn check(&self, service: &str) -> CheckerResult {
        let response = self.req_builder.try_clone().unwrap().send().await;
        match response {
            std::result::Result::Ok(response) => {
                if response.status() == self.expected_code {
                    CheckerResult::new(
                        service.to_string(),
                        CheckerStatus::Success,
                        format!("Service available with status {}", response.status()),
                    )
                } else {
                    CheckerResult::new(
                        service.to_string(),
                        CheckerStatus::Error,
                        format!("Service unavailable with status {}", response.status()),
                    )
                }
            }
            Err(err) => CheckerResult::new(
                service.to_string(),
                CheckerStatus::Error,
                format!("Service unavailable: {err}"),
            ),
        }
    }
}

impl TryFrom<&Yaml> for WebChecker {
    type Error = anyhow::Error;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let domain = match &data["domain"] {
            Yaml::String(d) if !d.is_empty() => d.clone(),
            _ => bail!(ConfigurationParseError::KeyNotFound(
                "domain",
                "at service of type web and cannot be empty"
            )),
        };

        let path = match &data["path"] {
            Yaml::String(p) if !p.is_empty() => Some(p.clone()),
            _ => None,
        };

        let expected_code = match data["expected_http_code"] {
            Yaml::Integer(http_code)
                if http_code >= u16::MIN as i64 && http_code <= u16::MAX as i64 =>
            {
                http_code as u16
            }
            _ => bail!(ConfigurationParseError::KeyNotFound(
                "expected_http_code",
                "at service of type web and be a valid HTTP code"
            )),
        };

        let http_code = match StatusCode::from_u16(expected_code) {
            Ok(code) => code,
            Err(_) => bail!(ConfigurationParseError::KeyNotFound(
                "expected_http_code",
                "at service of type web and be a valid HTTP code"
            )),
        };

        let headers = match &data["headers"] {
            Yaml::Hash(headers) => {
                let mut header_map = HeaderMap::new();
                for (key, value) in headers.iter() {
                    match (key, value) {
                        (Yaml::String(key), Yaml::String(value))
                            if !key.is_empty() && !value.is_empty() =>
                        {
                            let header_name = HeaderName::from_str(key);
                            let header_value = HeaderValue::from_str(value);
                            if header_name.is_err() || header_value.is_err() {
                                continue;
                            }
                            header_map.insert(header_name.unwrap(), header_value.unwrap());
                        }
                        _ => continue,
                    };
                }
                Some(header_map)
            }
            _ => None,
        };

        Ok(WebChecker::new(domain, path, http_code, headers))
    }
}

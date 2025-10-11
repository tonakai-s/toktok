use std::{str::FromStr, time::Duration};

use reqwest::{
    Client, RequestBuilder, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use yaml_rust2::Yaml;

use crate::{
    checker::{
        Checker,
        structs::{CheckerParserError, CheckerResult, CheckerStatus, CheckerType},
    },
    parser::{ConfigKey, keys::ConfigKeyInvalidFormat},
};

#[derive(Debug)]
pub struct WebCheckerBuilder {
    req_builder: RequestBuilder,
    expected_code: StatusCode,
}

impl WebCheckerBuilder {
    fn new(url: &str, expected_code: StatusCode) -> Self {
        Self {
            req_builder: Client::new().get(url),
            expected_code,
        }
    }

    fn headers(mut self, headers: HeaderMap) -> Self {
        self.req_builder = self.req_builder.headers(headers);
        self
    }

    fn timeout(mut self, timeout: Duration) -> Self {
        self.req_builder = self.req_builder.timeout(timeout);
        self
    }

    fn build(self) -> WebChecker {
        WebChecker {
            req_builder: self.req_builder,
            expected_code: self.expected_code,
        }
    }
}

#[derive(Debug)]
pub struct WebChecker {
    req_builder: RequestBuilder,
    expected_code: StatusCode,
}

impl WebChecker {
    pub fn builder(url: &str, expected_code: StatusCode) -> WebCheckerBuilder {
        WebCheckerBuilder::new(url, expected_code)
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
    type Error = CheckerParserError;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let url = match &data[ConfigKey::Url.as_ref()] {
            Yaml::String(d) if !d.is_empty() => d,
            _ => {
                return Err(CheckerParserError::KeyNotFoundAt(
                    ConfigKey::Url,
                    CheckerType::Web,
                ));
            }
        };

        let expected_http_code = match data[ConfigKey::ExpectedHttpCode.as_ref()] {
            Yaml::Integer(http_code)
                if http_code >= u16::MIN as i64 && http_code <= u16::MAX as i64 =>
            {
                StatusCode::from_u16(http_code as u16).map_err(|_| {
                    CheckerParserError::InvalidFormat(
                        ConfigKey::ExpectedHttpCode,
                        ConfigKeyInvalidFormat::new(ConfigKey::ExpectedHttpCode),
                    )
                })?
            }
            _ => {
                return Err(CheckerParserError::InvalidFormat(
                    ConfigKey::ExpectedHttpCode,
                    ConfigKeyInvalidFormat::new(ConfigKey::ExpectedHttpCode),
                ));
            }
        };

        let mut web_checker = WebChecker::builder(url, expected_http_code);

        if let Some(timeout) = Checker::timeout(data)? {
            web_checker = web_checker.timeout(timeout);
        }

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

        if let Some(headers) = headers {
            web_checker = web_checker.headers(headers);
        }

        Ok(web_checker.build())
    }
}

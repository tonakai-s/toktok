use std::str::FromStr;

use reqwest::{
    Client, RequestBuilder, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use yaml_rust2::Yaml;

use crate::checker::structs::{CheckerResult, CheckerStatus};

#[derive(Debug)]
pub struct WebChecker {
    req_builder: RequestBuilder,
    expected_code: StatusCode,
}

impl WebChecker {
    pub fn new(url: String, expected_code: StatusCode, headers: Option<HeaderMap>) -> Self {
        let client = Client::new().get(url).headers(headers.unwrap_or_default());

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
    type Error = String;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let url = match &data["url"] {
            Yaml::String(d) if !d.is_empty() => d.clone(),
            _ => {
                return Err(String::from(
                    "Key 'url' is mandatory for service of type web",
                ));
            }
        };

        let expected_http_code = match data["expected_http_code"] {
            Yaml::Integer(http_code)
                if http_code >= u16::MIN as i64 && http_code <= u16::MAX as i64 =>
            {
                StatusCode::from_u16(http_code as u16)
                    .map_err(|_| format!("{http_code} is not a valid HTTP code"))?
            }
            _ => return Err(String::from("Invalid ''")),
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

        Ok(WebChecker::new(url, expected_http_code, headers))
    }
}

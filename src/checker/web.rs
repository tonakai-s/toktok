use anyhow::bail;
use reqwest::StatusCode;
use yaml_rust2::{yaml::Hash, Yaml};

use crate::checker::structs::{CheckerResult, CheckerStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebChecker {
    domain: String,
    path: Option<String>,
    url: Option<String>,
    expected_code: StatusCode,
}

impl WebChecker {
    pub fn new(domain: String, path: Option<String>, expected_code: StatusCode) -> Self {
        let url = path.as_ref().map(|path| format!("{domain}{path}"));
        Self {
            domain,
            path,
            url,
            expected_code,
        }
    }

    fn url(&self) -> &str {
        if let Some(ref url) = self.url {
            url
        } else {
            self.domain.as_str()
        }
    }

    pub fn expected_code(&self) -> &StatusCode {
        &self.expected_code
    }

    pub async fn check(&self, service: &str) -> CheckerResult {
        let response = reqwest::get(self.url()).await;
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

impl TryFrom<&Hash> for WebChecker {
    type Error = anyhow::Error;
    fn try_from(value: &Hash) -> Result<Self, Self::Error> {
        let domain = match value.get(&Yaml::String("domain".into())) {
            Some(d) => d.as_str().unwrap().to_string(),
            None => bail!("'domain' is a mandatory field in web type service config"),
        };
        let path = value
            .get(&Yaml::String("path".into()))
            .map(|path| path.as_str().unwrap().to_string());
        let expected_code =
            match value.get(&Yaml::String("expected_http_code".into())) {
                Some(code) => {
                    if let Some(code) = code.as_i64()
                        && code >= u16::MIN as i64
                        && code <= u16::MAX as i64
                    {
                        code
                    } else {
                        bail!("'expected_http_code' must be a valid HTTP code")
                    }
                }
                None => bail!(
                    "'expected_http_code' is a mandatory field in web type service config"
                ),
            };
        let http_code = match StatusCode::from_u16(expected_code as u16) {
            Ok(code) => code,
            Err(_) => bail!("'expected_http_code' must be a valid HTTP code"),
        };

        Ok(WebChecker::new(domain, path, http_code))
    }
}
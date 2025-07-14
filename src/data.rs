use anyhow::bail;
use reqwest::StatusCode;
use yaml_rust2::{Yaml, yaml::Hash};

#[derive(Debug, Clone)]
pub enum Data {
    Web(WebData),
}

#[derive(Debug, Clone)]
pub struct WebData {
    domain: String,
    #[allow(dead_code)]
    path: Option<String>,
    url: Option<String>,
    expected_code: StatusCode,
}

impl TryFrom<&Hash> for Data {
    type Error = anyhow::Error;

    fn try_from(data_config: &Hash) -> Result<Self, Self::Error> {
        let service_type = match data_config.get(&Yaml::String("type".into())) {
            Some(t) => t.as_str().unwrap().to_lowercase(),
            None => bail!("'type' is mandatory field for a service"),
        };
        match service_type.as_str() {
            "web" => {
                let domain = match data_config.get(&Yaml::String("domain".into())) {
                    Some(d) => d.as_str().unwrap().to_string(),
                    None => bail!("'domain' is a mandatory field in web type service config"),
                };
                let path = data_config
                    .get(&Yaml::String("path".into()))
                    .map(|path| path.as_str().unwrap().to_string());
                let expected_code =
                    match data_config.get(&Yaml::String("expected_http_code".into())) {
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
                Ok(Data::Web(WebData::new(domain, path, http_code)))
            }
            _ => bail!("Type '{}' is not valid", service_type),
        }
    }
}

impl WebData {
    pub fn new(domain: String, path: Option<String>, expected_code: StatusCode) -> Self {
        let url = path.as_ref().map(|path| format!("{domain}{path}"));
        Self {
            domain,
            path,
            url,
            expected_code,
        }
    }

    pub fn url(&self) -> &str {
        if let Some(ref url) = self.url {
            url
        } else {
            self.domain.as_str()
        }
    }

    pub fn expected_code(&self) -> &StatusCode {
        &self.expected_code
    }
}

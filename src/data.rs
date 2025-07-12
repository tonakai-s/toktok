use reqwest::StatusCode;

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

impl WebData {
    pub fn new(domain: String, path: Option<String>, expected_code: StatusCode) -> Self {
        let url = path.as_ref().map(|path| format!("{}{}", domain, path));
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

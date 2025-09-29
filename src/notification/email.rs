use std::{fs, io::Read, path::PathBuf};

use anyhow::{Result, bail};
use lettre::{
    message::{header::ContentType, Mailbox}, transport::smtp::authentication::Credentials, Address, Message, SmtpTransport, Transport
};
use tracing::{event, span, Level};
use yaml_rust2::{Yaml, yaml::Hash};

use crate::{checker::structs::CheckerResult, notification::Notifier};

struct MailCredentials {
    user: String,
    pass: String
}

#[derive(Debug, Clone)]
pub struct MailNotifier {
    mailer: SmtpTransport,
    from: Mailbox,
    to: Mailbox,
    _cc: Option<String>,
    _bcc: Option<String>,
}
impl MailNotifier {
    fn new(credentials: Option<MailCredentials>, domain: String, from: String, to: String) -> Result<Self> {
        let mailer: SmtpTransport = if let Some(credentials) = credentials {
            SmtpTransport::relay(&domain)
                .unwrap()
                .credentials(
                    Credentials::new(credentials.user, credentials.pass)
                )
                .build()
        } else {
            SmtpTransport::builder_dangerous(&domain).build()
        };

        let from_address = match from.parse::<Address>() {
            Ok(addr) => addr,
            Err(err) => bail!("Error parsing the 'from' mail address, double check it: {}", err)
        };
        let to_address = match to.parse::<Address>() {
            Ok(addr) => addr,
            Err(err) => bail!("Error parsing the 'to' mail address, double check it: {}", err)
        };
        let from = Mailbox::new(None, from_address);
        let to = Mailbox::new(None, to_address);

        Ok(Self {
            mailer,
            from,
            to,
            _cc: None,
            _bcc: None,
        })
    }
}

impl Notifier for MailNotifier {
    fn notify(&self, exec_result: &CheckerResult) {
        let span = span!(Level::INFO, "MailNotifier::notify");
        let _enter = span.enter();

        let body = format!(
            "Hello, the service {} reported with status '{}' in the last verification: {}",
            exec_result.service_name, exec_result.status, exec_result.message
        );
        let email = Message::builder()
            .from(self.from.clone())
            .to(self.to.clone())
            .subject("Toktok Service Alert!")
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .unwrap();

        match self.mailer.send(&email) {
            Ok(_) => event!(
                Level::INFO,
                "Email notification sent successfully"
            ),
            Err(err) => event!(
                Level::ERROR,
                error = %err,
                "Error sending the Email notification"
            ),
        };
    }
}

impl TryFrom<&Hash> for MailNotifier {
    type Error = anyhow::Error;
    fn try_from(data: &Hash) -> Result<Self, Self::Error> {
        let smtp_domain = match data.get(&Yaml::String("smtp_domain".into())) {
            Some(domain) => domain.as_str().unwrap(),
            None => bail!("Key 'smtp_domain' is mandatory for 'mailer'"),
        };
        let from = match data.get(&Yaml::String("from".into())) {
            Some(from) => from.as_str().unwrap(),
            None => bail!("Key 'from' is mandatory for 'mailer'"),
        };
        let to = match data.get(&Yaml::String("to".into())) {
            Some(to) => to.as_str().unwrap(),
            None => bail!("Key 'to' is mandatory for 'mailer'"),
        };

        let credentials: Option<MailCredentials> = match data.get(&Yaml::String("smtp_credentials".into())) {
            Some(creds) => {
                let credentials_path = creds.as_str().unwrap();
                let credentials_path = PathBuf::from(credentials_path);
                if !credentials_path.exists() {
                    bail!("Path in 'smtp_credentials' does not exists");
                }

                let mut file = match fs::File::open(credentials_path) {
                    Ok(f) => f,
                    Err(err) => bail!("Unable to open the credentials file: {}", err),
                };
                let mut buff = String::new();
                match file.read_to_string(&mut buff) {
                    Ok(_) => (),
                    Err(_) => bail!("The credentials file must contain only valid UTF-8 characters"),
                }

                if buff.is_empty() {
                    bail!("Mailer credentials file is empty")
                }

                let mut buff_iter = buff.lines().take(2).into_iter();
                if let Some(username) = buff_iter.next()
                    && let Some(password) = buff_iter.next()
                {
                    Some(MailCredentials { user: username.into(), pass: password.into()})
                } else {
                    None
                }
            },
            None => None
        };

        MailNotifier::new(
            credentials,
            smtp_domain.into(),
            from.into(),
            to.into(),
        )
    }
}

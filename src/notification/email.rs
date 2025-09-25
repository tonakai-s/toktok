use std::{fs, io::Read, path::PathBuf};

use anyhow::bail;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use yaml_rust2::{Yaml, yaml::Hash};

use crate::{executor::ExecutionResult, notification::Notifier};

#[derive(Debug, Clone)]
pub struct MailNotifier {
    mailer: SmtpTransport,
    from: Mailbox,
    to: Mailbox,
    _cc: Option<String>,
    _bcc: Option<String>,
}
impl MailNotifier {
    fn new(user: String, pass: String, domain: String, from: String, to: String) -> Self {
        let creds = Credentials::new(user, pass);
        let mailer = SmtpTransport::relay(&domain)
            .unwrap()
            .credentials(creds)
            .build();

        let from = Mailbox::new(None, from.parse().unwrap());
        let to = Mailbox::new(None, to.parse().unwrap());
        Self {
            mailer,
            from,
            to,
            _cc: None,
            _bcc: None,
        }
    }
}

impl Notifier for MailNotifier {
    fn notify(&self, exec_result: &ExecutionResult) {
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
            Ok(_) => println!("Email sent successfully!"),
            Err(err) => println!("Error sending the email: {:#?}", err),
        };
    }
}

impl TryFrom<&Hash> for MailNotifier {
    type Error = anyhow::Error;
    fn try_from(data: &Hash) -> Result<Self, Self::Error> {
        let credentials_file_path = match data.get(&Yaml::String("smtp_credentials".into())) {
            Some(cred) => cred.as_str().unwrap(),
            None => bail!("Key 'smtp_credentials' is mandatory for 'mailer'"),
        };
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

        let creds_path = PathBuf::from(credentials_file_path);
        if !creds_path.exists() {
            bail!("Path in 'smtp_credentials' does not exists");
        }

        let mut file = match fs::File::open(creds_path) {
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
            return Ok(MailNotifier::new(
                username.into(),
                password.into(),
                smtp_domain.into(),
                from.into(),
                to.into(),
            ));
        } else {
            bail!(
                "Mailer credentials username and password cannot be empty.\nExpected file format: fist line = username, second line = password"
            )
        }
    }
}

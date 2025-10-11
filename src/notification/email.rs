use std::{fs, io::Read, path::PathBuf};

use lettre::{
    Address, Message, SmtpTransport, Transport,
    message::{Mailbox, MessageBuilder, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tracing::{Level, event, span};
use yaml_rust2::Yaml;

use crate::{
    checker::structs::CheckerResult,
    notification::{NotificationType, Notifier, error::NotificationParseError},
    parser::ConfigKey,
};

struct MailCredentials {
    user: String,
    pass: String,
}

#[derive(Debug, Clone)]
pub struct MailNotifier {
    mailer: SmtpTransport,
    base_msg_builder: MessageBuilder,
}
impl MailNotifier {
    fn try_new(
        domain: &str,
        from: &str,
        to: &str,
        credentials_path: Option<&str>,
    ) -> Result<MailNotifierBuilder, NotificationParseError> {
        MailNotifierBuilder::new(domain, from, to, credentials_path)
    }
}

impl Notifier for MailNotifier {
    fn notify(&self, exec_result: &CheckerResult) {
        let span = span!(Level::INFO, "MailNotifier::notify");
        let _enter = span.enter();

        let body = format!(
            r#"
<!DOCTYPE html>
<html>
  <head>
    <style>
      p {}
    </style>
  </head>
  <body>
    <h1>Toktok!</h1>
    <p>Hello, a service reported with an unexpected status through the last verification</p>
    <p>Service: {}</p>
    <p>Reported status: {}</p>
    <p>Message: {}</p>
  </body>
</html>
"#,
            "{margin: .5em 0 .5em 0; font-size: 16px;}",
            exec_result.service_name,
            exec_result.status,
            exec_result.message
        );

        let email = self.base_msg_builder.clone().body(body);
        if let Err(e) = email {
            event!(Level::ERROR, error = %e, "Error building the email message");
            return;
        }

        match self.mailer.send(&email.unwrap()) {
            Ok(_) => event!(Level::INFO, "Email notification sent successfully"),
            Err(err) => event!(
                Level::ERROR,
                error = %err,
                "Error sending the Email notification"
            ),
        };
    }
}

struct MailNotifierBuilder {
    smtp_transport: SmtpTransport,
    msg_builder: MessageBuilder,
}
impl MailNotifierBuilder {
    fn new(
        domain: &str,
        from: &str,
        to: &str,
        credentials_path: Option<&str>,
    ) -> Result<Self, NotificationParseError> {
        let credentials = MailNotifierBuilder::credentials(credentials_path)?;
        let smtp_transport = MailNotifierBuilder::smtp_transport(credentials, domain)?;
        let msg_builder = MailNotifierBuilder::msg_builder(from, to)?;

        Ok(Self {
            smtp_transport,
            msg_builder,
        })
    }

    fn credentials(
        credentials_path_str: Option<&str>,
    ) -> Result<Option<MailCredentials>, NotificationParseError> {
        if credentials_path_str.is_none() {
            return Ok(None);
        }

        let credentials_path = PathBuf::from(credentials_path_str.unwrap());
        if !credentials_path.exists() {
            return Err(NotificationParseError::InternalParse(format!(
                "The path in {} does not exists.",
                ConfigKey::SmtpCredentials
            )));
        }

        let mut file = fs::File::open(credentials_path).map_err(|e| {
            NotificationParseError::UnableToOpenFile(credentials_path_str.unwrap().to_string(), e)
        })?;

        let mut buff = String::new();
        file.read_to_string(&mut buff).map_err(|e| {
            NotificationParseError::UnableToReadFile(credentials_path_str.unwrap().to_string(), e)
        })?;

        if buff.is_empty() {
            return Err(NotificationParseError::InternalParse(
                "Email credentials cannot be empty".to_string(),
            ));
        }

        let mut buff_iter = buff.lines().take(2);
        if let Some(username) = buff_iter.next()
            && let Some(password) = buff_iter.next()
        {
            Ok(Some(MailCredentials {
                user: username.into(),
                pass: password.into(),
            }))
        } else {
            Ok(None)
        }
    }

    fn smtp_transport(
        credentials: Option<MailCredentials>,
        domain: &str,
    ) -> Result<SmtpTransport, NotificationParseError> {
        if let Some(credentials) = credentials {
            let transport_builder = SmtpTransport::relay(domain).map_err(|e| {
                NotificationParseError::InternalBuild(format!(
                    "The program was unable to build a SMTP relay with the provided domain: {e}"
                ))
            })?;

            Ok(transport_builder
                .credentials(Credentials::new(credentials.user, credentials.pass))
                .build())
        } else {
            Ok(SmtpTransport::builder_dangerous(domain).build())
        }
    }

    fn msg_builder(from: &str, to: &str) -> Result<MessageBuilder, NotificationParseError> {
        let from_box = Mailbox::new(
            None,
            from.parse::<Address>().map_err(|e| {
                NotificationParseError::InternalParse(format!(
                    "Error parsing the '{}' mail address, double check it: {e}",
                    ConfigKey::MailFrom
                ))
            })?,
        );
        let to_box = Mailbox::new(
            None,
            to.parse::<Address>().map_err(|e| {
                NotificationParseError::InternalParse(format!(
                    "Error parsing the '{}' mail address, double check it: {e}",
                    ConfigKey::MailTo
                ))
            })?,
        );

        Ok(Message::builder()
            .from(from_box)
            .to(to_box)
            .subject("Toktok Service Alert!")
            .header(ContentType::TEXT_HTML))
    }

    fn cc(mut self, list: Vec<String>) -> Result<Self, NotificationParseError> {
        let boxes = list
            .iter()
            .map(|addr| {
                Ok(Mailbox::new(
                    None,
                    addr.parse::<Address>().map_err(|e| {
                        NotificationParseError::InternalParse(format!(
                            "The email address '{addr}' at cc list is not valid: {e}"
                        ))
                    })?,
                ))
            })
            .collect::<Result<Vec<Mailbox>, NotificationParseError>>()?;

        for m_box in boxes.into_iter() {
            self.msg_builder = self.msg_builder.cc(m_box);
        }

        Ok(self)
    }

    fn bcc(mut self, list: Vec<String>) -> Result<Self, NotificationParseError> {
        let boxes = list
            .iter()
            .map(|addr| {
                Ok(Mailbox::new(
                    None,
                    addr.parse::<Address>().map_err(|e| {
                        NotificationParseError::InternalParse(format!(
                            "The email address '{addr}' at bcc list is not valid: {e}"
                        ))
                    })?,
                ))
            })
            .collect::<Result<Vec<Mailbox>, NotificationParseError>>()?;

        for m_box in boxes.into_iter() {
            self.msg_builder = self.msg_builder.bcc(m_box);
        }

        Ok(self)
    }

    fn build(self) -> MailNotifier {
        MailNotifier {
            mailer: self.smtp_transport,
            base_msg_builder: self.msg_builder,
        }
    }
}

impl TryFrom<&Yaml> for MailNotifier {
    type Error = NotificationParseError;

    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let smtp_domain = match &data[ConfigKey::SmtpDomain.as_ref()] {
            Yaml::String(domain) => domain,
            _ => {
                return Err(NotificationParseError::KeyNotFoundAt(
                    ConfigKey::SmtpDomain,
                    NotificationType::Email,
                ));
            }
        };

        let from = match &data[ConfigKey::MailFrom.as_ref()] {
            Yaml::String(from) => from,
            _ => {
                return Err(NotificationParseError::KeyNotFoundAt(
                    ConfigKey::MailFrom,
                    NotificationType::Email,
                ));
            }
        };

        let to = match &data[ConfigKey::MailTo.as_ref()] {
            Yaml::String(to) => to,
            _ => {
                return Err(NotificationParseError::KeyNotFoundAt(
                    ConfigKey::MailTo,
                    NotificationType::Email,
                ));
            }
        };

        let credentials_path = match &data[ConfigKey::SmtpCredentials.as_ref()] {
            creds if !creds.is_badvalue() => creds.as_str(),
            _ => None,
        };

        let mut mail_builder = MailNotifier::try_new(smtp_domain, from, to, credentials_path)?;

        let cc: Option<Vec<String>> = match &data[ConfigKey::MailCc.as_ref()] {
            Yaml::Array(cc_list)
                if !cc_list.is_empty() && cc_list.iter().all(|addr| addr.as_str().is_some()) =>
            {
                Some(
                    cc_list
                        .iter()
                        .map(|addr| addr.as_str().unwrap().trim().to_string())
                        .collect(),
                )
            }
            Yaml::BadValue => None,
            _ => {
                return Err(NotificationParseError::InternalParse(format!(
                    "Key '{}' must have a YAML Array format",
                    ConfigKey::MailCc
                )));
            }
        };
        if let Some(cc) = cc {
            mail_builder = mail_builder.cc(cc)?;
        }

        let bcc: Option<Vec<String>> = match &data[ConfigKey::MailBcc.as_ref()] {
            Yaml::Array(bcc_list)
                if !bcc_list.is_empty() && bcc_list.iter().all(|addr| addr.as_str().is_some()) =>
            {
                Some(
                    bcc_list
                        .iter()
                        .map(|addr| addr.as_str().unwrap().trim().to_string())
                        .collect(),
                )
            }
            Yaml::BadValue => None,
            _ => {
                return Err(NotificationParseError::InternalParse(format!(
                    "Key '{}' must have a YAML Array format",
                    ConfigKey::MailBcc
                )));
            }
        };
        if let Some(bcc) = bcc {
            mail_builder = mail_builder.bcc(bcc)?;
        }

        Ok(mail_builder.build())
    }
}

use std::{fs, io::Read, path::PathBuf};

use lettre::{
    Address, Message, SmtpTransport, Transport,
    message::{Mailbox, MessageBuilder, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tracing::{Level, event, span};
use yaml_rust2::Yaml;

use crate::{checker::structs::CheckerResult, notification::Notifier};

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
        credentials: Option<MailCredentials>,
        domain: String,
        from: String,
        to: String,
        cc: Option<Vec<String>>,
        bcc: Option<Vec<String>>,
    ) -> Result<Self, String> {
        let mailer: SmtpTransport = if let Some(credentials) = credentials {
            SmtpTransport::relay(&domain)
                .map_err(|e| {
                    format!(
                        "The program was unable to build a SMTP relay with the provided domain: {e}"
                    )
                })?
                .credentials(Credentials::new(credentials.user, credentials.pass))
                .build()
        } else {
            SmtpTransport::builder_dangerous(&domain).build()
        };

        let from_box = Mailbox::new(
            None,
            from.parse::<Address>().map_err(|e| {
                format!("Error parsing the 'from' mail address, double check it: {e}")
            })?,
        );
        let to_box = Mailbox::new(
            None,
            to.parse::<Address>().map_err(|e| {
                format!("Error parsing the 'to' mail address, double check it: {e}")
            })?,
        );

        let mut base_msg_builder = Message::builder()
            .from(from_box)
            .to(to_box)
            .subject("Toktok Service Alert!")
            .header(ContentType::TEXT_PLAIN);

        let cc_boxes = cc.and_then(|addrs| {
            addrs
                .iter()
                .map(|addr| {
                    Ok(Mailbox::new(
                        None,
                        addr.parse::<Address>().map_err(|e| {
                            format!("The email address {addr} at cc list is not valid: {e}")
                        })?,
                    ))
                })
                .collect::<Result<Vec<Mailbox>, String>>()
                .ok()
        });
        if let Some(boxes) = cc_boxes {
            for mbox in boxes.into_iter() {
                base_msg_builder = base_msg_builder.cc(mbox);
            }
        }

        let bcc_boxes = bcc.and_then(|addrs| {
            addrs
                .iter()
                .map(|addr| {
                    Ok(Mailbox::new(
                        None,
                        addr.parse::<Address>().map_err(|e| {
                            format!("The email address {addr} at bcc list is not valid: {e}")
                        })?,
                    ))
                })
                .collect::<Result<Vec<Mailbox>, String>>()
                .ok()
        });
        if let Some(boxes) = bcc_boxes {
            for mbox in boxes.into_iter() {
                base_msg_builder = base_msg_builder.bcc(mbox);
            }
        }

        Ok(Self {
            mailer,
            base_msg_builder,
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

impl TryFrom<&Yaml> for MailNotifier {
    type Error = String;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let smtp_domain = match &data["smtp_domain"] {
            Yaml::String(domain) => domain,
            _ => return Err(String::from("Key 'smtp_domain' is mandatory for mailer")),
        };

        let from = match &data["from"] {
            Yaml::String(from) => from,
            _ => return Err(String::from("Key 'from' is mandatory for mailer")),
        };

        let to = match &data["to"] {
            Yaml::String(to) => to,
            _ => return Err(String::from("Key 'to' is mandatory for mailer")),
        };

        let cc: Option<Vec<String>> = match &data["cc"] {
            Yaml::String(cc_list) if !cc_list.is_empty() => Some(
                cc_list
                    .split(',')
                    .map(|addr| addr.trim().to_string())
                    .collect(),
            ),
            _ => None,
        };
        let bcc: Option<Vec<String>> = match &data["bcc"] {
            Yaml::String(bcc_list) if !bcc_list.is_empty() => Some(
                bcc_list
                    .split(',')
                    .map(|addr| addr.trim().to_string())
                    .collect(),
            ),
            _ => None,
        };

        let credentials: Option<MailCredentials> = match &data["smtp_credentials"] {
            creds if !creds.is_badvalue() && creds.as_str().is_some() => {
                let credentials_path = creds.as_str().unwrap();
                let credentials_path = PathBuf::from(credentials_path);
                if !credentials_path.exists() {
                    return Err(String::from("Path in 'smtp_credentials' does not exists"));
                }

                let mut file = fs::File::open(credentials_path)
                    .map_err(|e| format!("Unable to open the credentials file: {e}"))?;

                let mut buff = String::new();
                file.read_to_string(&mut buff).map_err(|e| {
                    format!("The credentials file must contain only valid UTF-8 characters: {e}")
                })?;

                if buff.is_empty() {
                    return Err(String::from("Mailer credentials file is empty"));
                }

                let mut buff_iter = buff.lines().take(2);
                if let Some(username) = buff_iter.next()
                    && let Some(password) = buff_iter.next()
                {
                    Some(MailCredentials {
                        user: username.into(),
                        pass: password.into(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        MailNotifier::try_new(
            credentials,
            smtp_domain.into(),
            from.into(),
            to.into(),
            cc,
            bcc,
        )
    }
}

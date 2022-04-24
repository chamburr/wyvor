use crate::{config::CONFIG, ApiResult};

use handlebars::Handlebars;
use lazy_static::lazy_static;
use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use rust_embed::RustEmbed;
use serde_json::json;
use std::str::FromStr;

lazy_static! {
    static ref HBS: Handlebars<'static> = {
        let mut hbs = Handlebars::new();
        hbs.register_embed_templates::<Templates>().unwrap();
        hbs
    };
}

#[derive(RustEmbed)]
#[folder = "templates"]
#[include = "*.hbs"]
struct Templates;

#[derive(Debug)]
pub struct Client(AsyncSmtpTransport<Tokio1Executor>);

impl Client {
    async fn send(&self, address: &str, subject: &str, body: String) -> ApiResult<()> {
        if !CONFIG.smtp_enabled {
            return Ok(());
        }

        let message = Message::builder()
            .from(Mailbox::new(
                None,
                Address::from_str(CONFIG.smtp_sender.as_str())?,
            ))
            .to(Mailbox::new(None, Address::from_str(address)?))
            .subject(subject)
            .body(body)?;

        self.0.send(message).await?;

        Ok(())
    }
}

impl Client {
    pub fn new() -> Self {
        let tls = if CONFIG.smtp_tls {
            Tls::Required(TlsParameters::new(CONFIG.smtp_host.clone()).unwrap())
        } else {
            Tls::None
        };

        let credentials = Credentials::new(CONFIG.smtp_user.clone(), CONFIG.smtp_password.clone());

        let client =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(CONFIG.smtp_host.as_str())
                .port(CONFIG.smtp_port)
                .tls(tls)
                .credentials(credentials)
                .build();

        Self(client)
    }

    pub async fn send_welcome(&self, address: &str, name: &str, token: &str) -> ApiResult<()> {
        let link = format!("{}/verify?token={}", CONFIG.base_uri, token);
        let body = HBS.render("welcome", &json!({ "name": name, "link": link }))?;

        self.send(address, "Welcome to Wyvor", body).await?;

        Ok(())
    }

    pub async fn send_verify(&self, address: &str, name: &str, token: &str) -> ApiResult<()> {
        let link = format!("{}/verify?token={}", CONFIG.base_uri, token);
        let body = HBS.render("verify", &json!({ "name": name, "link": link}))?;

        self.send(address, "Verify your Wyvor account", body)
            .await?;

        Ok(())
    }

    pub async fn send_reset(
        &self,
        address: &str,
        name: &str,
        token: &str,
        id: i64,
    ) -> ApiResult<()> {
        let link = format!("{}/reset?token={}&id={}", CONFIG.base_uri, token, id);
        let body = HBS.render("reset", &json!({ "name": name, "link": link }))?;

        self.send(address, "Reset your Wyvor account", body).await?;

        Ok(())
    }

    pub async fn send_password(&self, address: &str, name: &str) -> ApiResult<()> {
        let body = HBS.render("password", &json!({ "name": name }))?;

        self.send(address, "Wyvor account password changed", body)
            .await?;

        Ok(())
    }

    pub async fn send_invite(&self, address: &str, name: &str, space: &str) -> ApiResult<()> {
        let link = format!("{}/dashboard", CONFIG.base_uri);
        let body = HBS.render(
            "invite",
            &json!({ "name": name, "space": space, "link": link }),
        )?;

        self.send(address, "Wyvor space invitation", body).await?;

        Ok(())
    }
}

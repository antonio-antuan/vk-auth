use anyhow::{Error, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::time::Duration;
use url::Url;

const START_PATTERN: &str = "location.href='";
const END_PATTERN: &str = "';</script>";

#[derive(Debug, Clone)]
pub enum ParseError {
    NoLoginForm,
    NoFormAction,
    InvalidFormInputField,
    InvalidRedirectData,
    AuthorizationFailed,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoLoginForm => {
                write!(f, "NoLoginForm")
            }
            ParseError::NoFormAction => {
                write!(f, "NoFormAction")
            }
            ParseError::InvalidFormInputField => {
                write!(f, "InvalidFormInputField")
            }
            ParseError::InvalidRedirectData => {
                write!(f, "InvalidRedirectData: expected data not found on page")
            }
            ParseError::AuthorizationFailed => {
                write!(f, "AuthorizationFailed: invalid authorization data")
            }
        }?;
        Ok(())
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub struct AccessToken {
    access_token: String,
    expires_in: Duration,
    user_id: String,
}

impl AccessToken {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    pub fn expires_in(&self) -> Duration {
        self.expires_in
    }
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
}

#[derive(Debug, Clone)]
pub struct Authorizer {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct AuthorizerBuilder {
    client: Option<Client>,
}

impl AuthorizerBuilder {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<Authorizer> {
        let client = self
            .client
            .unwrap_or(reqwest::Client::builder().cookie_store(true).build()?);
        Ok(Authorizer { client })
    }
}

impl Default for AuthorizerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Authorizer {
    pub fn builder() -> AuthorizerBuilder {
        AuthorizerBuilder::new()
    }

    pub async fn get_token(
        &self,
        api_id: &str,
        email_or_phone: &str,
        password: &str,
    ) -> Result<AccessToken> {
        let initial_resp = self
            .client
            .get(format!(
                "https://oauth.vk.com/oauth/authorize?client_id={}&scope=0&response_type=token",
                api_id
            ))
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(initial_resp.as_str());
        let form = doc
            .select(&Selector::parse("form").unwrap())
            .next()
            .ok_or(ParseError::NoLoginForm)?;
        let url = form
            .value()
            .attr("action")
            .ok_or(ParseError::NoFormAction)?;
        let mut data = HashMap::new();
        data.insert("email", email_or_phone);
        data.insert("pass", password);
        data.insert("expire", "0");
        for node in form.children() {
            let val = node.value();
            if !val.is_element() {
                continue;
            }
            let inp = node.value().as_element().unwrap();
            if inp.name() != "input" {
                continue;
            }
            if inp.attr("type").unwrap() == "hidden" {
                data.insert(
                    inp.attr("name").ok_or(ParseError::InvalidFormInputField)?,
                    inp.attr("value").ok_or(ParseError::InvalidFormInputField)?,
                );
            }
        }
        let resp = self
            .client
            .post(url)
            .form(&data)
            .send()
            .await?
            .text()
            .await?;
        get_token_from_page(resp.as_str()).map_err(|e| match e.downcast_ref::<ParseError>() {
            Some(parse_err) => match parse_err {
                ParseError::NoLoginForm => e,
                ParseError::NoFormAction => e,
                ParseError::InvalidFormInputField => e,
                ParseError::InvalidRedirectData => {
                    if doc
                        .select(
                            &Selector::parse(r#"form[action*="https://login.vk.com"]"#).unwrap(),
                        )
                        .next()
                        .is_some()
                    {
                        Error::from(ParseError::AuthorizationFailed)
                    } else {
                        e
                    }
                }
                ParseError::AuthorizationFailed => e,
            },
            None => e,
        })
    }
}

fn get_token_from_page(resp: &str) -> Result<AccessToken> {
    let sfound = resp
        .find(START_PATTERN)
        .ok_or(ParseError::InvalidRedirectData)?;
    let efound = resp
        .rfind(END_PATTERN)
        .ok_or(ParseError::InvalidRedirectData)?;
    let redirect_url = resp[sfound + START_PATTERN.len()..efound].into();
    let query: HashMap<_, _> = form_urlencoded::parse(
        Url::parse(redirect_url)?
            .fragment()
            .ok_or(ParseError::InvalidRedirectData)?
            .as_bytes(),
    )
    .into_owned()
    .collect();
    Ok(AccessToken {
        access_token: query
            .get("access_token")
            .ok_or(ParseError::InvalidRedirectData)?
            .into(),
        expires_in: Duration::from_secs(
            query
                .get("expires_in")
                .ok_or(ParseError::InvalidRedirectData)?
                .parse()?,
        ),
        user_id: query
            .get("user_id")
            .ok_or(ParseError::InvalidRedirectData)?
            .into(),
    })
}

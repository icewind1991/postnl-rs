use crate::data::Package;
use err_derive::Error;

use crate::auth::{new_token, refresh_token, AccessToken};
use reqwest::header;
use std::sync::Mutex;

pub use crate::auth::Token;

mod auth;
pub mod data;
mod dimensions;
mod formatted;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Failed to initialize the client: {}", _0)]
    ClientInitialization(#[error(source, no_from)] reqwest::Error),
    #[error(display = "Network error: {}", _0)]
    NetworkError(#[error(source)] reqwest::Error),
    #[error(display = "Error while parsing json result: {}", _0)]
    JSONError(#[error(source)] serde_json::Error),
    #[error(display = "Failed to retrieve request validation token for login")]
    NoRequestValidationToken,
    #[error(display = "Failed to validate login request: {}", _0)]
    ValidateFailure(String),
    #[error(display = "Failed to retrieve static url for login")]
    NoStaticUrl,
    #[error(display = "Failed to get token: {}", _0)]
    FailedToken(String),
    #[error(display = "Invalid credentials")]
    Authentication,
    #[error(display = "Connection blocked by PostNL, try again in a while")]
    Blocked,
}

type Result<T> = std::result::Result<T, Error>;

pub struct PostNL {
    username: String,
    password: String,
    token: Mutex<Option<Token>>,
    client: reqwest::Client,
}

// https://jouw.postnl.nl/web/api/default/inbox ?
static SHIPMENTS_URL: &str = "https://jouw.postnl.nl/web/api/shipments";
static _PROFILE_URL: &str = "https://jouw.postnl.nl/web/api/profile";
static _LETTERS_URL: &str = "https://jouw.postnl.nl/web/api/letters";
static _VALIDATE_LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters/validation";

impl PostNL {
    pub fn new(username: impl ToString, password: impl ToString) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Api-Version", header::HeaderValue::from_static("4.18"));
        headers.insert(
            "User-Agent",
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; rv:68.0) Gecko/20100101 Firefox/68.0",
            ),
        );

        Ok(PostNL {
            username: username.to_string(),
            password: password.to_string(),
            token: Mutex::default(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    /// Ensure that we have valid credentials
    async fn authenticate(&self) -> Result<AccessToken> {
        let token = self.token.lock().unwrap().take();

        let new_token = match token {
            Some(old_token) => {
                refresh_token(&self.client, old_token, &self.username, &self.password).await?
            }
            None => new_token(&self.username, &self.password).await?,
        };

        let access_token = new_token.access.clone();

        self.token.lock().unwrap().replace(new_token);

        Ok(access_token)
    }

    /// Get the authentication token for caching
    pub async fn get_token(&self) -> Result<Token> {
        self.authenticate().await?;
        Ok(self.token.lock().unwrap().as_ref().unwrap().clone())
    }

    /// Set a cached token
    pub fn set_token(&self, token: Token) {
        self.token.lock().unwrap().replace(token);
    }

    pub async fn check_credentials(&self) -> Result<()> {
        self.authenticate().await?;
        Ok(())
    }

    pub async fn get_packages(&self) -> Result<Vec<Package>> {
        let token = self.authenticate().await?;

        Ok(self
            .client
            .get(SHIPMENTS_URL)
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?)
    }
}

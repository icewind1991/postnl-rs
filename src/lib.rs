use crate::data::Package;
use err_derive::Error;
use parse_display::Display;
use serde::Deserialize;

use std::sync::Mutex;
use std::time::{Duration, Instant};
use reqwest::Response;

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
    #[error(display = "Invalid credentials")]
    Authentication,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize)]
struct RawToken {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Display, Clone, Debug)]
struct AccessToken(String);

#[derive(Display, Clone, Debug)]
struct RefreshToken(String);

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "RawToken")]
struct Token {
    access: AccessToken,
    refresh: RefreshToken,
    expires: Instant,
}

impl Token {
    pub fn need_refresh(&self) -> bool {
        self.expires < Instant::now()
    }
}

impl From<RawToken> for Token {
    fn from(raw: RawToken) -> Self {
        Token {
            access: AccessToken(raw.access_token),
            refresh: RefreshToken(raw.refresh_token),
            expires: Instant::now() + Duration::from_secs(raw.expires_in - 15),
        }
    }
}

pub struct PostNL {
    username: String,
    password: String,
    token: Mutex<Option<Token>>,
    client: reqwest::Client,
}

static AUTHENTICATE_URL: &str = "https://jouw.postnl.nl/mobile/token";
static SHIPMENTS_URL: &str = "https://jouw.postnl.nl/mobile/api/shipments";
static _PROFILE_URL: &str = "https://jouw.postnl.nl/mobile/api/profile";
static _LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters";
static _VALIDATE_LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters/validation";

impl PostNL {
    pub fn new(username: impl ToString, password: impl ToString) -> Result<Self> {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert("api-version", header::HeaderValue::from_static("4.16"));

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
            Some(old_token) => self.refresh_token(old_token).await?,
            None => self.new_token().await?,
        };

        let access_token = new_token.access.clone();

        self.token.lock().unwrap().replace(new_token);

        Ok(access_token)
    }

    async fn new_token(&self) -> Result<Token> {
        let response: Response = self.client.post(AUTHENTICATE_URL)
            .form(&[
                ("grant_type", "password"),
                ("client_id", "pwAndroidApp"),
                ("username", &self.username),
                ("password", &self.password),
            ])
            .send()
            .await?;
        if response.status().is_client_error() {
            Err(Error::Authentication)
        } else {
            Ok(response.json().await?)
        }
    }

    async fn refresh_token(&self, token: Token) -> Result<Token> {
        if token.need_refresh() {
            let response: Response = self.client.post(AUTHENTICATE_URL)
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", &token.refresh.0)
                ])
                .send()
                .await?;

            if response.status().is_success() {
                Ok(response.json().await?)
            } else {
                self.new_token().await
            }
        } else {
            Ok(token)
        }
    }

    pub async fn check_credentials(&self) -> Result<()> {
        self.authenticate().await?;
        Ok(())
    }

    pub async fn get_packages(&self) -> Result<Vec<Package>> {
        let token = self.authenticate().await?;

        Ok(self.client.get(SHIPMENTS_URL)
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

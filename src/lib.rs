use crate::data::Package;
use maplit::hashmap;
use parse_display::Display;
use reqwest::header;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::time::{Duration, Instant};

pub mod data;

#[derive(Debug)]
pub enum Error {
    ClientInitializationFailed(reqwest::Error),
    NetworkError(reqwest::Error),
    JSONError(reqwest::Error),
}

impl Error {
    pub fn client(err: reqwest::Error) -> Self {
        Error::ClientInitializationFailed(err)
    }

    pub fn network(err: reqwest::Error) -> Self {
        Error::NetworkError(err)
    }

    pub fn json(err: reqwest::Error) -> Self {
        Error::JSONError(err)
    }
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

#[derive(Clone, Debug)]
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
    token: RefCell<Option<Token>>,
    client: Client,
}

static AUTHENTICATE_URL: &str = "https://jouw.postnl.nl/mobile/token";
static SHIPMENTS_URL: &str = "https://jouw.postnl.nl/mobile/api/shipments";
static _PROFILE_URL: &str = "https://jouw.postnl.nl/mobile/api/profile";
static _LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters";
static _VALIDATE_LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters/validation";

impl PostNL {
    pub fn new(username: String, password: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("api-version", header::HeaderValue::from_static("4.16"));
        headers.insert(
            "X-Client-Library",
            header::HeaderValue::from_static("postnl-rs"),
        );

        Ok(PostNL {
            username,
            password,
            token: RefCell::default(),
            client: Client::builder()
                .default_headers(headers)
                .build()
                .map_err(Error::client)?,
        })
    }

    /// Ensure that we have valid credentials
    fn authenticate(&self) -> Result<AccessToken> {
        let mut mut_ref = self.token.borrow_mut();
        let option = mut_ref.deref_mut();
        match option {
            Some(token) if token.need_refresh() => *token = self.refresh_token(token)?,
            None => *option = Some(self.new_token()?),
            _ => {}
        };

        Ok(option.as_ref().unwrap().access.clone())
    }

    fn new_token(&self) -> Result<Token> {
        let mut response = self
            .client
            .request(Method::POST, AUTHENTICATE_URL)
            .form(&hashmap! {
                "grant_type" => "password",
                "client_id" => "pwAndroidApp",
                "username" => &self.username,
                "password" => &self.password,
            })
            .send()
            .map_err(Error::network)?;
        let raw: RawToken = response.json().map_err(Error::json)?;
        Ok(raw.into())
    }

    fn refresh_token(&self, old_token: &Token) -> Result<Token> {
        let mut response = self
            .client
            .request(Method::POST, AUTHENTICATE_URL)
            .form(&hashmap! {
                "grant_type" => "refresh_token".to_string(),
                "refresh_token" => old_token.refresh.to_string()
            })
            .send()
            .map_err(Error::network)?;
        if response.status() == StatusCode::OK {
            let raw: RawToken = response.json().map_err(Error::json)?;
            Ok(raw.into())
        } else {
            self.new_token()
        }
    }

    pub fn get_packages(&self) -> Result<Vec<Package>> {
        let token = self.authenticate()?;
        let mut response = self
            .client
            .request(Method::GET, SHIPMENTS_URL)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .map_err(Error::network)?;

        response.json::<Vec<Package>>().map_err(Error::JSONError)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use crate::data::Package;
use maplit::hashmap;
use parse_display::Display;
use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use surf::{Client, Response};

pub mod data;

#[derive(Debug)]
pub enum Error {
    NetworkError(surf::Exception),
    JSONError(std::io::Error),
}

impl From<surf::Exception> for Error {
    fn from(err: surf::Exception) -> Self {
        Error::NetworkError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
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
}

static AUTHENTICATE_URL: &str = "https://jouw.postnl.nl/mobile/token";
static SHIPMENTS_URL: &str = "https://jouw.postnl.nl/mobile/api/shipments";
static _PROFILE_URL: &str = "https://jouw.postnl.nl/mobile/api/profile";
static _LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters";
static _VALIDATE_LETTERS_URL: &str = "https://jouw.postnl.nl/mobile/api/letters/validation";

impl PostNL {
    pub fn new(username: impl ToString, password: impl ToString) -> Self {
        PostNL {
            username: username.to_string(),
            password: password.to_string(),
            token: Mutex::default(),
        }
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
        Ok(Client::new()
            .post(AUTHENTICATE_URL)
            .set_header("api-version", "4.16")
            .body_form(&hashmap! {
                "grant_type" => "password",
                "client_id" => "pwAndroidApp",
                "username" => &self.username,
                "password" => &self.password,
            })
            .unwrap()
            .recv_json()
            .await?)
    }

    async fn refresh_token(&self, token: Token) -> Result<Token> {
        if token.need_refresh() {
            let mut response: Response = Client::new()
                .post(AUTHENTICATE_URL)
                .set_header("api-version", "4.16")
                .body_form(&hashmap! {
                    "grant_type" => "refresh_token",
                    "refresh_token" => &token.refresh.0
                })
                .unwrap()
                .await?;
            if response.status().is_success() {
                Ok(response.body_json().await?)
            } else {
                self.new_token().await
            }
        } else {
            Ok(token)
        }
    }

    pub async fn get_packages(&self) -> Result<Vec<Package>> {
        let token = self.authenticate().await?;
        let auth = format!("Bearer {}", token);
        Ok(Client::new()
            .get(SHIPMENTS_URL)
            .set_header("api-version", "4.16")
            .set_header("Authorization", &auth)
            .set_header("Content-Type", "application/json")
            .recv_json()
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

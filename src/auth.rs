use crate::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use parse_display::Display;
use rand::Rng;
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use serde::export::TryFrom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use url::Url;

static LOGIN_URL: &str = "https://jouw.postnl.nl/identity/Account/Login";
static AUTHORIZE_URL: &str = "https://jouw.postnl.nl/identity/connect/authorize";
static TOKEN_URL: &str = "https://jouw.postnl.nl/identity/connect/token";

#[derive(Deserialize)]
struct RawToken {
    access_token: String,
    id_token: String,
    expires_in: i64,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawTokenResponse {
    Error(ErrorResponse),
    Ok(RawToken),
}

#[derive(Display, Clone, Debug, Serialize, Deserialize)]
pub struct AccessToken(String);

#[derive(Display, Clone, Debug, Serialize, Deserialize)]
pub struct RefreshToken(String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub(crate) access: AccessToken,
    pub(crate) id_token: RefreshToken,
    pub(crate) expires: DateTime<Utc>,
}

impl Token {
    pub fn need_refresh(&self) -> bool {
        self.expires < Utc::now()
    }
}

impl TryFrom<RawTokenResponse> for Token {
    type Error = Error;

    fn try_from(raw: RawTokenResponse) -> Result<Self> {
        match raw {
            RawTokenResponse::Ok(token) => Ok(Token {
                access: AccessToken(token.access_token),
                id_token: RefreshToken(token.id_token),
                expires: Utc::now() + Duration::seconds(token.expires_in - 15),
            }),
            RawTokenResponse::Error(err) => Err(Error::FailedToken(err.error)),
        }
    }
}

pub async fn new_token(username: &str, password: &str) -> Result<Token> {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .build()?;

    let response: Response = client.get(LOGIN_URL).send().await?;
    let body = response.text().await?;

    let request_token_regex =
        regex::Regex::new(r#"__RequestVerificationToken.* value="([^"]*)"#).unwrap();
    let request_verification_token = match request_token_regex.captures(&body) {
        Some(captures) => captures[1].to_string(),
        None => return Err(Error::NoRequestValidationToken),
    };

    let static_url_regex = regex::Regex::new(r#"src="(/static/[a-z0-9]+)""#).unwrap();
    let static_url = match static_url_regex.captures(&body) {
        Some(captures) => captures[1].to_string(),
        None => return Err(Error::NoStaticUrl),
    };

    let random_sensor_data = hex_random(22);

    let data = format!(
        r#"{{"sensor_data":"'{}'{}"}}"#,
        random_sensor_data,
        include_str!("sensordata.txt")
    );

    let response: Response = client
        .post(&format!("https://jouw.postnl.nl/{}", static_url))
        .body(data)
        .send()
        .await?;

    let result: ValidateResponse = response.json().await?;
    if !result.success {
        return Err(Error::ValidateFailure(
            result
                .error
                .unwrap_or_else(|| "no error provided".to_string()),
        ));
    }

    let response: Response = client
        .post(LOGIN_URL)
        .form(&[
            (
                "__RequestVerificationToken",
                request_verification_token.as_str(),
            ),
            ("ReturnUrl", ""),
            ("Username", &username),
            ("Password", &password),
        ])
        .send()
        .await?;

    if let Some(header) = response
        .headers()
        .get("location")
        .and_then(|header| header.to_str().ok())
    {
        if let Ok(location) = Url::parse(header) {
            let location_query_pairs = location
                .query_pairs()
                .into_owned()
                .collect::<HashMap<String, String>>();

            if location_query_pairs.get("botdetected").map(|s| s.as_str()) == Some("true") {
                return Err(Error::Blocked);
            }
        }
    }

    let mut hasher = Sha256::new();
    let code_verifier = hex_random(64);
    hasher.input(code_verifier.as_bytes());
    let code_challenge = base64::encode(hasher.result())
        .replace('+', "-")
        .replace('\\', "_")
        .replace('=', "");
    let state_value = hex_random(64);

    let response: Response = client
        .get(AUTHORIZE_URL)
        .query(&[
            ("client_id", "pwb-web"),
            ("audience", "poa-profiles-api"),
            ("scope", "openid profile email poa-profiles-api pwb-web-api"),
            ("response_type", "code"),
            ("code_challenge_method", "S256"),
            ("code_challenge", &code_challenge),
            ("prompt", "none"),
            ("state", &state_value),
            ("redirect_uri", "https://jouw.postnl.nl/silent-renew.html"),
            ("ui_locales", "nl_NL"),
        ])
        .send()
        .await?;

    let location_header = Url::parse(
        response
            .headers()
            .get("location")
            .ok_or_else(|| Error::ValidateFailure("No redirect provided".to_string()))?
            .to_str()
            .map_err(|_| Error::ValidateFailure("Invalid redirect provided".to_string()))?,
    )
    .map_err(|_| Error::ValidateFailure("Invalid redirect provided".to_string()))?;

    let mut location_query_pairs = location_header
        .query_pairs()
        .into_owned()
        .collect::<HashMap<String, String>>();

    if let Some(err) = location_query_pairs.remove("error") {
        return Err(Error::ValidateFailure(err));
    }

    let code = location_query_pairs
        .remove("code")
        .ok_or_else(|| Error::ValidateFailure("No code provided".to_string()))?;

    let response: Response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "pwb-web"),
            ("code", &code),
            ("code_verifier", &code_verifier),
            ("redirect_uri", "https://jouw.postnl.nl/silent-renew.html"),
        ])
        .send()
        .await?;

    let raw_token: RawTokenResponse = response.json().await?;
    Token::try_from(raw_token)
}

pub async fn refresh_token(
    client: &Client,
    token: Token,
    username: &str,
    password: &str,
) -> Result<Token> {
    if token.need_refresh() {
        let response: Response = client
            .post(TOKEN_URL)
            .form(&[("grant_type", "id_token"), ("id_token", &token.id_token.0)])
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            new_token(username, password).await
        }
    } else {
        Ok(token)
    }
}

#[derive(Deserialize)]
struct ValidateResponse {
    success: bool,
    error: Option<String>,
}

fn hex_random(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length).fold(String::with_capacity(length), |mut buff, _| {
        let num = rng.gen_range(0, 15);
        buff.push(std::char::from_digit(num, 16).unwrap());
        buff
    })
}

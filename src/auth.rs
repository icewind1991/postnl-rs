use crate::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use parse_display::Display;
use rand::Rng;
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use serde::export::{PhantomData, TryFrom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;
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

pub trait AuthState {}

pub struct New;

impl AuthState for New {}

pub struct LoggedIn;

impl AuthState for LoggedIn {}

pub struct AuthHandler<State: AuthState> {
    client: Client,
    state: PhantomData<State>,
}

impl AuthHandler<New> {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .redirect(Policy::none())
            .build()?;

        Ok(AuthHandler {
            client,
            state: PhantomData,
        })
    }

    pub async fn login(self, username: &str, password: &str) -> Result<AuthHandler<LoggedIn>> {
        let verification_token = self.verify_login().await?;
        self.do_login(username, password, &verification_token)
            .await?;

        Ok(AuthHandler::<LoggedIn> {
            client: self.client,
            state: PhantomData,
        })
    }

    /// Get the info needed to verify that we are "not a bot"
    async fn get_request_verification_info(&self) -> Result<VerificationInfo> {
        let response: Response = self.client.get(LOGIN_URL).send().await?;
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

        Ok({
            VerificationInfo {
                token: request_verification_token,
                url: static_url,
            }
        })
    }

    /// "Proof" that we are "not a bot", returning the verification token
    async fn verify_login(&self) -> Result<String> {
        let verification_info = self.get_request_verification_info().await?;
        let random_sensor_data = hex_random(22);
        let data = format!(
            r#"{{"sensor_data":"'{}'{}"}}"#,
            random_sensor_data,
            include_str!("sensordata.txt")
        );

        let response: Response = self
            .client
            .post(&format!("https://jouw.postnl.nl/{}", verification_info.url))
            .body(data)
            .send()
            .await?;

        let result: ValidateResponse = response.json().await?;
        if !result.success {
            return Err(Error::VerificationFailure(
                result
                    .error
                    .unwrap_or_else(|| "no error provided".to_string()),
            ));
        }
        Ok(verification_info.token)
    }

    /// Send the actual login request, setting the cookies
    async fn do_login(
        &self,
        username: &str,
        password: &str,
        verification_token: &str,
    ) -> Result<()> {
        let response: Response = self
            .client
            .post(LOGIN_URL)
            .form(&[
                ("__RequestVerificationToken", verification_token),
                ("ReturnUrl", ""),
                ("Username", &username),
                ("Password", &password),
            ])
            .send()
            .await?;

        if let Some(location_header) = get_redirect_url(&response) {
            if location_header
                .query_pairs()
                .find(|(key, _)| key == "botdetected")
                .map(|(_, value)| bool::from_str(value.as_ref()).unwrap_or_default())
                == Some(true)
            {
                return Err(Error::Blocked);
            }
        }

        Ok(())
    }
}

impl AuthHandler<LoggedIn> {
    pub async fn generate_token(&self) -> Result<Token> {
        let code = self
            .do_authorization(AuthorizationParams::new(), false)
            .await?;
        let raw_token = self.get_token_from_code(code).await?;
        Token::try_from(raw_token)
    }

    /// Get the authorization code using the stored login cookies
    async fn do_authorization(
        &self,
        auth_params: AuthorizationParams,
        prompt: bool,
    ) -> Result<AuthorizationCode> {
        let response: Response = self
            .client
            .get(AUTHORIZE_URL)
            .query(&[
                ("client_id", "pwb-web"),
                ("audience", "poa-profiles-api"),
                ("scope", "openid profile email poa-profiles-api pwb-web-api"),
                ("response_type", "code"),
                ("code_challenge_method", "S256"),
                ("code_challenge", &auth_params.code_challenge),
                ("prompt", if prompt { "prompt" } else { "none" }),
                ("state", &auth_params.state),
                ("redirect_uri", "https://jouw.postnl.nl/silent-renew.html"),
                ("ui_locales", "nl_NL"),
            ])
            .send()
            .await?;

        let location_header = get_redirect_url(&response)
            .ok_or(Error::AuthorizationFailure("No or invalid redirect url"))?;

        let mut location_query_pairs = location_header.query_pairs().collect::<HashMap<_, _>>();

        if let Some(err) = location_query_pairs.remove("error") {
            return Err(Error::VerificationFailure(err.to_string()));
        }

        let code = location_query_pairs
            .remove("code")
            .ok_or(Error::AuthorizationFailure("No code provided"))?;

        Ok(AuthorizationCode {
            code: code.to_string(),
            code_verifier: auth_params.code_verifier,
        })
    }

    /// Get the auth token using the authorization code
    async fn get_token_from_code(&self, code: AuthorizationCode) -> Result<RawTokenResponse> {
        let response: Response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", "pwb-web"),
                ("code", &code.code),
                ("code_verifier", &code.code_verifier),
                ("redirect_uri", "https://jouw.postnl.nl/silent-renew.html"),
            ])
            .send()
            .await?;

        Ok(response.json().await?)
    }
}

struct AuthorizationParams {
    code_verifier: String,
    state: String,
    code_challenge: String,
}

struct AuthorizationCode {
    code: String,
    code_verifier: String,
}

impl AuthorizationParams {
    pub fn new() -> Self {
        let mut hasher = Sha256::new();
        let code_verifier = hex_random(64);
        hasher.input(code_verifier.as_bytes());
        let code_challenge = base64::encode(hasher.result())
            .replace('+', "-")
            .replace('\\', "_")
            .replace('=', "");
        let state = hex_random(64);

        AuthorizationParams {
            code_challenge,
            code_verifier,
            state,
        }
    }
}

struct VerificationInfo {
    url: String,
    token: String,
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

fn get_redirect_url(response: &Response) -> Option<Url> {
    response
        .headers()
        .get("location")
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| Url::parse(header_str).ok())
}

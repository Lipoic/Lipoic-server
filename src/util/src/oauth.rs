use database::model::auth::user::ConnectType;
use reqwest::Error;
use serde::Deserialize;
use urlencoding::encode;

use crate::util::get_redirect_uri_by_path;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO: &str = "https://www.googleapis.com/oauth2/v1/userinfo?alt=json";

const FACEBOOK_AUTH_URL: &str = "https://www.facebook.com/dialog/oauth";
const FACEBOOK_TOKEN_URL: &str = "https://graph.facebook.com/v14.0/oauth/access_token";
const FACEBOOK_USER_INFO: &str = "https://graph.facebook.com/v14.0";

pub struct OAuthData<'a> {
  pub account_type: &'a ConnectType,
  pub client_secret: &'a String,
  pub client_id: &'a String,
  pub issuer: &'a String,
  pub redirect_path: &'a str,
}

#[derive(Deserialize)]
pub struct AccessTokenInfo {
    pub access_token: String,
    pub expires_in: i32,
    /// Appears only in google OAuth
    #[serde(skip_deserializing)]
    pub scope: String,
    pub token_type: String,
    /// Appears only in google OAuth
    #[serde(skip_deserializing)]
    pub id_token: String,
}

#[derive(Deserialize)]
pub struct GoogleAccountInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

#[derive(Deserialize)]
pub struct FacebookAccountInfo {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
    pub email: String,
    pub picture: FacebookAccountPicture,
}


#[derive(Deserialize)]
pub struct FacebookAccountPicture {
    pub data: FacebookAccountPictureData,
}

#[derive(Deserialize)]
pub struct FacebookAccountPictureData {
    pub height: i32,
    pub is_silhouette: bool,
    pub url: String,
    pub width: i32,
}

#[derive(Deserialize)]
pub struct OAuthAccountInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub picture: String,
    pub verified_email: bool,
}

impl OAuthAccountInfo{
    fn from_google(google_account_info: GoogleAccountInfo) -> Self {
        OAuthAccountInfo {
            id: google_account_info.id,
            name: google_account_info.name,
            email: google_account_info.email,
            picture: google_account_info.picture,
            verified_email: google_account_info.verified_email,
        }
    }

    fn from_facebook(facebook_account_info: FacebookAccountInfo) -> Self {
        OAuthAccountInfo {
            id: facebook_account_info.id,
            name: facebook_account_info.name,
            email: facebook_account_info.email,
            picture: facebook_account_info.picture.data.url,
            verified_email: true,
        }
    }
}


impl OAuthData<'_> {
    pub fn new<'a>(
        account_type:&'a ConnectType,
        client_secret: &'a String,
        client_id: &'a String,
        issuer: &'a String,
        redirect_path: &'a str,
    ) -> OAuthData<'a> {
        OAuthData {
            account_type,
            client_secret,
            client_id,
            issuer,
            redirect_path,
        }
    }

    /// get google oauth url
    ///
    /// return one url [`String`]
    pub fn get_auth_url(&self) -> String {
        let scope = match self.account_type {
            ConnectType::Google => 
                 encode("https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email"),
                 ConnectType::Facebook => 
                 encode("public_profile,email"),
        };
        
        let auth_url = match self.account_type {
            ConnectType::Google => GOOGLE_AUTH_URL,
            ConnectType::Facebook => FACEBOOK_AUTH_URL,
        };

        let redirect_uri = get_redirect_uri_by_path(self.issuer, self.redirect_path);

        format!(
            "{}?client_id={}&response_type=code&scope={}&redirect_uri={}",
            auth_url,
            self.client_id,
            scope,
            encode(redirect_uri.as_ref())
        )
    }

    /// get access token info by code
    ///
    /// return [`AccessTokenInfo`]
    pub async fn authorization_code(&self, code: String) -> Result<AccessTokenInfo, Error> {
        let mut form_data = vec![
            ("client_id", self.client_id.clone()),
            ("client_secret", self.client_secret.clone()),
            ("code", code),
            (
                "redirect_uri",
                get_redirect_uri_by_path(self.issuer, self.redirect_path),
            ),
        ];

        if matches!(self.account_type, ConnectType::Google) {
            form_data.push(("grant_type", "authorization_code".to_string()));
        }

        let token_url = match self.account_type {
            ConnectType::Google => GOOGLE_TOKEN_URL,
            ConnectType::Facebook => FACEBOOK_TOKEN_URL,
        };

        let response = reqwest::Client::new()
            .post(token_url)
            .form(&form_data)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .send()
            .await?;

        response.json::<AccessTokenInfo>().await
    }

}

impl AccessTokenInfo {
    /// request google user info
    ///
    /// return [`GoogleAccountInfo`]
    pub async fn get_google_user_info(
        &self,
    ) -> Result<GoogleAccountInfo, Box<dyn std::error::Error>> {
        let response = reqwest::Client::new()
            .get(GOOGLE_USER_INFO)
            .bearer_auth(self.access_token.clone())
            .send()
            .await?;

        Ok(response.json::<GoogleAccountInfo>().await?)
    }

    /// request facebook user info
    ///
    /// return [`FacebookAccountInfo`]
    pub async fn get_facebook_user_info(
        &self,
    ) -> Result<FacebookAccountInfo, Box<dyn std::error::Error>> {
        let response = reqwest::Client::new()
            .get(format!("{}/me?fields=id,first_name,last_name,name,email,picture&access_token={}", FACEBOOK_USER_INFO,self.access_token.clone()))
            .send()
            .await?;

        Ok(response.json::<FacebookAccountInfo>().await?)
    }

    pub async fn get_account_info(&self,account_type:&ConnectType) -> Result<OAuthAccountInfo, Box<dyn std::error::Error>> {
       match account_type {
           ConnectType::Google => Ok(OAuthAccountInfo::from_google(self.get_google_user_info().await?)),
           ConnectType::Facebook =>  Ok(OAuthAccountInfo::from_facebook(self.get_facebook_user_info().await?)),
       }
    }
}

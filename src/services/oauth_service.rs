use serde::{Deserialize, Serialize};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    basic::BasicClient, 
    reqwest::async_http_client,
    AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use reqwest;
use std::error::Error;
use base64::{Engine as _, engine::general_purpose};

/// Informações do usuário retornadas pelo OAuth provider
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub oauth_id: String, // ID único do provider (sub no Google, sub no Apple)
    pub email_verified: bool,
}

/// Resposta do Google UserInfo
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,           // ID único do Google
    email: String,
    name: Option<String>,
    picture: Option<String>,
    email_verified: bool,
}

/// Resposta do Apple UserInfo (formato ID Token JWT)
#[derive(Debug, Deserialize)]
struct AppleIdToken {
    sub: String,           // ID único da Apple
    email: String,
    email_verified: Option<String>, // "true" ou "false" como string
}

/// Configuração do OAuth Client
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
}

/// Serviço de autenticação OAuth
pub struct OAuthService {
    google_client: Option<BasicClient>,
    apple_client: Option<BasicClient>,
}

impl OAuthService {
    /// Criar novo serviço OAuth
    pub fn new(
        google_config: Option<OAuthConfig>,
        apple_config: Option<OAuthConfig>,
    ) -> Result<Self, Box<dyn Error>> {
        let google_client = if let Some(config) = google_config {
            Some(BasicClient::new(
                ClientId::new(config.client_id),
                Some(ClientSecret::new(config.client_secret)),
                AuthUrl::new(config.auth_url)?,
                Some(TokenUrl::new(config.token_url)?),
            ).set_redirect_uri(RedirectUrl::new(config.redirect_url)?))
        } else {
            None
        };

        let apple_client = if let Some(config) = apple_config {
            Some(BasicClient::new(
                ClientId::new(config.client_id),
                Some(ClientSecret::new(config.client_secret)),
                AuthUrl::new(config.auth_url)?,
                Some(TokenUrl::new(config.token_url)?),
            ).set_redirect_uri(RedirectUrl::new(config.redirect_url)?))
        } else {
            None
        };

        Ok(Self {
            google_client,
            apple_client,
        })
    }

    /// Gerar URL de autorização do Google
    pub fn get_google_auth_url(&self) -> Result<(String, String), String> {
        let client = self.google_client.as_ref()
            .ok_or("Google OAuth not configured")?;

        // Removendo PKCE temporariamente para desenvolvimento
        // Em produção, deve-se armazenar o pkce_verifier no Redis/sessão
        // let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            // .set_pkce_challenge(pkce_challenge)  // Comentado para desenvolvimento
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    /// Gerar URL de autorização da Apple
    pub fn get_apple_auth_url(&self) -> Result<(String, String), String> {
        let client = self.apple_client.as_ref()
            .ok_or("Apple OAuth not configured")?;

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("name".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    /// Trocar código de autorização por token (Google)
    pub async fn exchange_google_code(&self, code: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let client = self.google_client.as_ref()
            .ok_or("Google OAuth not configured")?;

        log::info!("🔄 Exchanging Google authorization code...");
        
        let token_result = match client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await {
                Ok(token) => token,
                Err(e) => {
                    log::error!("❌ Google token exchange failed: {:?}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Google token exchange error: {}", e)
                    )));
                }
            };

        log::info!("✅ Google token exchange successful");
        Ok(token_result.access_token().secret().clone())
    }

    /// Trocar código de autorização por token (Apple)
    pub async fn exchange_apple_code(&self, code: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let client = self.apple_client.as_ref()
            .ok_or("Apple OAuth not configured")?;

        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await?;

        // Apple retorna um ID token JWT, não access token tradicional
        Ok(token_result.access_token().secret().clone())
    }

    /// Buscar informações do usuário do Google
    pub async fn get_google_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        
        log::info!("📥 Fetching Google user info...");
        
        // Usando v3 da API que retorna o campo 'sub' consistentemente
        let response = client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("❌ Failed to get Google user info. Status: {}, Body: {}", status, error_text);
            return Err(format!("Failed to get user info: {} - {}", status, error_text).into());
        }

        let response_text = response.text().await?;
        log::info!("📄 Google user info response: {}", response_text);
        
        let google_user: GoogleUserInfo = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("❌ Failed to parse Google user info: {}", e);
                format!("Failed to parse user info: {}", e)
            })?;

        log::info!("✅ Google user info fetched successfully: {}", google_user.email);

        Ok(OAuthUserInfo {
            email: google_user.email,
            name: google_user.name,
            picture: google_user.picture,
            oauth_id: google_user.sub,
            email_verified: google_user.email_verified,
        })
    }

    /// Buscar informações do usuário da Apple
    /// Apple retorna as informações no ID Token (JWT)
    pub async fn get_apple_user_info(&self, id_token: &str) -> Result<OAuthUserInfo, Box<dyn Error + Send + Sync>> {
        // Decodificar o JWT (sem verificar assinatura por simplicidade)
        // Em produção, você deve verificar a assinatura com as chaves públicas da Apple
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid ID token format".into());
        }

        // Decodificar o payload (parte do meio)
        let payload = general_purpose::STANDARD.decode(parts[1])
            .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(parts[1]))
            .map_err(|e| format!("Failed to decode ID token: {}", e))?;

        let apple_token: AppleIdToken = serde_json::from_slice(&payload)?;

        let email_verified = apple_token.email_verified
            .map(|v| v == "true")
            .unwrap_or(false);

        Ok(OAuthUserInfo {
            email: apple_token.email,
            name: None, // Apple não retorna nome no ID token por padrão
            picture: None, // Apple não fornece foto de perfil
            oauth_id: apple_token.sub,
            email_verified,
        })
    }

    /// Fluxo completo: Google
    pub async fn authenticate_google(&self, code: String) -> Result<OAuthUserInfo, Box<dyn Error + Send + Sync>> {
        let access_token = self.exchange_google_code(code).await?;
        self.get_google_user_info(&access_token).await
    }

    /// Fluxo completo: Apple
    pub async fn authenticate_apple(&self, code: String, id_token: Option<String>) -> Result<OAuthUserInfo, Box<dyn Error + Send + Sync>> {
        // Apple pode retornar o id_token diretamente ou podemos trocar o code
        let token = if let Some(id_token) = id_token {
            id_token
        } else {
            self.exchange_apple_code(code).await?
        };

        self.get_apple_user_info(&token).await
    }
}

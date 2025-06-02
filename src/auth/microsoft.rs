use anyhow::{Context, Result, anyhow};
use log::{debug, error, info, trace, warn};
use oauth2::{AuthUrl, ClientId, CsrfToken, RedirectUrl, Scope, TokenUrl, basic::BasicClient};
use std::env;
use tokio::sync::oneshot;
use tokio::task;

use super::constants::{MS_AUTH_URL, MS_TOKEN_URL, REDIRECT_URI};

/// Starts a local server to receive the OAuth redirect and extract the code
fn start_local_server(tx: tokio::sync::oneshot::Sender<Result<String>>) {
    use anyhow::anyhow;
    use log::{debug, error, info, warn};
    use std::net::SocketAddr;
    use tiny_http::{Response, Server};
    use url::Url;

    let addr: SocketAddr = match "127.0.0.1:8080".parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = tx.send(Err(anyhow!("Failed to parse address: {}", e)));
            return;
        }
    };
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to start local HTTP server: {e}");
            let _ = tx.send(Err(anyhow!(
                "Failed to start local HTTP server on {}: {}",
                addr,
                e
            )));
            return;
        }
    };
    info!("Local server listening on {addr}");
    info!("Waiting for redirect from Microsoft...");

    match server.recv() {
        Ok(rq) => {
            let url_str = format!("http://localhost:8080{}", rq.url());
            debug!("Received request: {url_str}");

            let response_text;
            if let Ok(url) = Url::parse(&url_str) {
                let code = url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned());
                let error = url
                    .query_pairs()
                    .find(|(key, _)| key == "error")
                    .map(|(_, value)| value.into_owned());
                let error_description = url
                    .query_pairs()
                    .find(|(key, _)| key == "error_description")
                    .map(|(_, value)| value.into_owned());

                if let Some(code) = code {
                    debug!("Received authorization code.");
                    response_text =
                        "Authentication successful! You can close this window.".to_string();
                    let _ = tx.send(Ok(code));
                } else if let Some(error) = error {
                    let description =
                        error_description.unwrap_or_else(|| "No description provided.".to_string());
                    error!("OAuth error received: {error} - {description}");
                    response_text = format!(
                        "Authentication failed: {error} - {description}. Please close this window."
                    );
                    let _ = tx.send(Err(anyhow!("OAuth error: {} - {}", error, description)));
                } else {
                    warn!("Received request without 'code' or 'error' parameter.");
                    response_text =
                        "Received unexpected request. Please close this window.".to_string();
                    let _ = tx.send(Err(anyhow!("Invalid redirect request received")));
                }
            } else {
                error!("Failed to parse redirect URL: {url_str}");
                response_text = "Error processing request. Please close this window.".to_string();
                let _ = tx.send(Err(anyhow!("Failed to parse redirect URL")));
            }

            let response = Response::from_string(response_text).with_status_code(200);
            if let Err(e) = rq.respond(response) {
                error!("Failed to send response to browser: {e}");
            }
            debug!("Local server responded and is shutting down.");
        }
        Err(e) => {
            error!("Local server failed to receive request: {e}");
            let _ = tx.send(Err(anyhow!("Local server error: {}", e)));
        }
    }
    // Server automatically stops after handling one request
}

/// Exchanges the authorization code for an access token
async fn exchange_code_for_token(
    oauth_client: &oauth2::basic::BasicClient,
    code: String,
) -> Result<String> {
    use anyhow::Context;
    use oauth2::{AuthorizationCode, TokenResponse};
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .context("Failed to exchange authorization code for token")?;
    Ok(token_result.access_token().secret().clone())
}

/// Get a Microsoft OAuth token using the authorization code flow with a local server
pub async fn get_microsoft_token() -> Result<String> {
    let client_id = env::var("MS_CLIENT_ID").context(
        "MS_CLIENT_ID environment variable not set. Make sure .env file is present and loaded.",
    )?;

    debug!("Creating OAuth client with client ID: {client_id}");

    let redirect_url =
        RedirectUrl::new(REDIRECT_URI.to_string()).context("Invalid redirect URI")?;

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        None, // No client secret for public clients
        AuthUrl::new(MS_AUTH_URL.to_string()).context("Invalid Microsoft Auth URL")?,
        Some(TokenUrl::new(MS_TOKEN_URL.to_string()).context("Invalid Microsoft Token URL")?),
    )
    .set_redirect_uri(redirect_url.clone());

    // Generate the authorization URL
    debug!(
        "Generating authorization URL with scopes: XboxLive.signin, offline_access and prompt=login"
    );
    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("XboxLive.signin".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        // .add_extra_param("prompt", "login")
        .url();

    // Channel to receive the authorization code from the local server
    let (tx, rx) = oneshot::channel::<Result<String>>();

    // Start the local server in a blocking thread
    let server_handle = task::spawn_blocking(move || {
        start_local_server(tx);
    });

    // Open the browser for the user to log in
    info!("Opening browser for Microsoft authentication...");
    webbrowser::open(auth_url.as_str()).context("Failed to open web browser for authentication")?;
    info!("Please complete the login in your browser. Waiting for authorization code...");

    // Wait for the server thread to send the code
    let code = rx
        .await
        .context("Authentication process cancelled or failed")??;

    // Ensure the server task finished
    server_handle.await.context("Server task panicked")?;

    if code.is_empty() {
        error!("Received empty authorization code");
        return Err(anyhow::anyhow!("Received empty authorization code"));
    }

    info!("Exchanging authorization code for access token...");
    debug!("Authorization code length: {}", code.len());

    // Exchange authorization code for access token
    let token = exchange_code_for_token(&oauth_client, code).await?;
    debug!("Successfully received access token");
    trace!("Access token length: {}", token.len());
    Ok(token)
}

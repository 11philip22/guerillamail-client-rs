//! GuerrillaMail async client implementation.

use crate::{Error, Message, Result};
use regex::Regex;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, HOST, ORIGIN, REFERER,
    USER_AGENT,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Async client for GuerrillaMail temporary email service.
///
/// Use [`Client::new`] for defaults or [`Client::builder`] for custom settings
/// like proxies, TLS behavior, and a custom user agent.
#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    api_token: String,
    proxy: Option<String>,
    user_agent: String,
    ajax_url: String,
}

impl Client {
    /// Create a builder for configuring the client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Create a new GuerrillaMail client.
    ///
    /// Connects to GuerrillaMail and retrieves the API token and available domains.
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        ClientBuilder::new().build().await
    }

    /// Get the proxy URL if one was configured.
    ///
    /// Returns `None` when no proxy was set on the builder.
    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    /// Create a temporary email address.
    ///
    /// # Arguments
    /// * `alias` - The email alias (part before @)
    ///
    /// # Returns
    /// The full email address assigned by GuerrillaMail
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::new().await?;
    /// let email = client.create_email("myalias").await?;
    /// println!("{email}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_email(&self, alias: &str) -> Result<String> {
        let params = [("f", "set_email_user")];
        let form = [
            ("email_user", alias),
            ("lang", "en"),
            ("site", "guerrillamail.com"),
            ("in", " Set cancel"),
        ];

        let response: serde_json::Value = self
            .http
            .post(&self.ajax_url)
            .query(&params)
            .form(&form)
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        response
            .get("email_addr")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(Error::TokenParse)
    }

    /// Get messages for an email address.
    ///
    /// # Arguments
    /// * `email` - The full email address
    ///
    /// # Returns
    /// A list of messages in the inbox
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::new().await?;
    /// let email = client.create_email("myalias").await?;
    /// let messages = client.get_messages(&email).await?;
    /// for msg in messages {
    ///     println!("{}: {}", msg.mail_from, msg.mail_subject);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_messages(&self, email: &str) -> Result<Vec<Message>> {
        let response = self.get_api("check_email", email, None).await?;

        let messages = response
            .get("list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<Message>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(messages)
    }

    /// Fetch the full content of a specific email.
    ///
    /// # Arguments
    /// * `email` - The full email address
    /// * `mail_id` - The message ID to fetch
    ///
    /// # Returns
    /// The full email details including the body
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::new().await?;
    /// let email = client.create_email("myalias").await?;
    /// let messages = client.get_messages(&email).await?;
    /// if let Some(msg) = messages.first() {
    ///     let details = client.fetch_email(&email, &msg.mail_id).await?;
    ///     println!("{}", details.mail_body);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_email(&self, email: &str, mail_id: &str) -> Result<crate::EmailDetails> {
        let response = self.get_api("fetch_email", email, Some(mail_id)).await?;
        serde_json::from_value(response).map_err(|_| Error::TokenParse)
    }

    /// Delete/forget an email address.
    ///
    /// # Arguments
    /// * `email` - The full email address to delete
    ///
    /// # Returns
    /// `true` if deletion was successful
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::new().await?;
    /// let email = client.create_email("myalias").await?;
    /// let ok = client.delete_email(&email).await?;
    /// println!("{ok}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_email(&self, email: &str) -> Result<bool> {
        let alias = Self::extract_alias(email);
        let params = [("f", "forget_me")];
        let form = [("site", "guerrillamail.com"), ("in", alias)];

        let response = self
            .http
            .post(&self.ajax_url)
            .query(&params)
            .form(&form)
            .headers(self.headers())
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Common GET API request pattern.
    async fn get_api(
        &self,
        function: &str,
        email: &str,
        email_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let alias = Self::extract_alias(email);
        let timestamp = Self::timestamp();

        let mut params = vec![
            ("f", function.to_string()),
            ("site", "guerrillamail.com".to_string()),
            ("in", alias.to_string()),
            ("_", timestamp),
        ];

        if let Some(id) = email_id {
            params.insert(1, ("email_id", id.to_string()));
        }

        if function == "check_email" {
            params.insert(1, ("seq", "1".to_string()));
        }

        let mut headers = self.headers();
        headers.remove(CONTENT_TYPE);

        self.http
            .get(&self.ajax_url)
            .query(&params)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Extract alias from email address.
    fn extract_alias(email: &str) -> &str {
        email.split('@').next().unwrap_or(email)
    }

    /// Generate timestamp for cache-busting.
    fn timestamp() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
    }

    /// Build headers for API requests.
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static("www.guerrillamail.com"));
        if let Ok(value) = HeaderValue::from_str(&self.user_agent) {
            headers.insert(USER_AGENT, value);
        }
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("ApiToken {}", self.api_token)).unwrap(),
        );
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_static("https://www.guerrillamail.com"),
        );
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://www.guerrillamail.com/"),
        );
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers.insert("Priority", HeaderValue::from_static("u=0"));
        headers
    }
}

const BASE_URL: &str = "https://www.guerrillamail.com";
const AJAX_URL: &str = "https://www.guerrillamail.com/ajax.php";
const USER_AGENT_VALUE: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";

/// Builder for configuring a GuerrillaMail client.
///
/// Start with [`Client::builder`] to override defaults.
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    proxy: Option<String>,
    danger_accept_invalid_certs: bool,
    user_agent: String,
    ajax_url: String,
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    ///
    /// Defaults:
    /// - No proxy
    /// - `danger_accept_invalid_certs = true`
    /// - Default user agent
    /// - Default GuerrillaMail AJAX endpoint
    pub fn new() -> Self {
        Self {
            proxy: None,
            danger_accept_invalid_certs: true,
            user_agent: USER_AGENT_VALUE.to_string(),
            ajax_url: AJAX_URL.to_string(),
        }
    }

    /// Set a proxy URL (e.g., "http://127.0.0.1:8080").
    ///
    /// This uses reqwest's proxy support for all requests.
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Control whether to accept invalid TLS certificates (default: true).
    ///
    /// Set this to `false` for stricter TLS validation.
    pub fn danger_accept_invalid_certs(mut self, value: bool) -> Self {
        self.danger_accept_invalid_certs = value;
        self
    }

    /// Override the default user agent string.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Override the AJAX endpoint URL.
    ///
    /// Useful for testing or when GuerrillaMail changes its endpoint.
    pub fn ajax_url(mut self, ajax_url: impl Into<String>) -> Self {
        self.ajax_url = ajax_url.into();
        self
    }

    /// Build the client and fetch initial API token + domains.
    ///
    /// This performs a network request to GuerrillaMail to bootstrap the session.
    ///
    /// # Examples
    /// ```no_run
    /// # use guerrillamail::Client;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), guerrillamail::Error> {
    /// let client = Client::builder()
    ///     .user_agent("my-app/1.0")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<Client> {
        let mut builder =
            reqwest::Client::builder().danger_accept_invalid_certs(self.danger_accept_invalid_certs);

        if let Some(proxy_url) = &self.proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }

        // Enable cookie store to persist session between requests
        let http = builder.cookie_store(true).build()?;

        // Fetch the main page to get API token and domains
        let response = http.get(BASE_URL).send().await?.text().await?;

        // Parse API token: api_token : 'xxxxxxxx'
        let token_re = Regex::new(r"api_token\s*:\s*'(\w+)'").unwrap();
        let api_token = token_re
            .captures(&response)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or(Error::TokenParse)?;

        Ok(Client {
            http,
            api_token,
            proxy: self.proxy,
            user_agent: self.user_agent,
            ajax_url: self.ajax_url,
        })
    }
}

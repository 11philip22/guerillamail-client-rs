//! # GuerrillaMail Client
//! Asynchronous wrapper around the GuerrillaMail disposable email HTTP API, providing simple methods to create, poll, and delete temporary inboxes from Rust using [`Client`] and [`ClientBuilder`].
//!
//! ## Audience and uses
//! For Rust developers who need throwaway addresses in integration tests, demos, or automation scripts without running mail infrastructure: configure with [`ClientBuilder`], obtain an address, poll for messages ([`Message`]), then discard the inbox when done.
//!
//! ## Runtime requirements
//! Async-only; run inside a Tokio (v1) runtime. HTTP calls use `reqwest`, so ensure the chosen Tokio features (`rt-multi-thread` or `current_thread`) are available in your application.
//!
//! ## Out of scope
//! Not a general-purpose mail client, SMTP sender, or durable mailbox. It only proxies the GuerrillaMail service and inherits its availability, spam filtering, and retention limits.
//!
//! ## Errors
//! All network calls surface transport and non-2xx statuses as [`Error::Request`]; shape or content issues become [`Error::ResponseParse`] or [`Error::Json`]. The crate-wide [`Result`] alias wraps these errors.
//!
//! ## Example
//! ```no_run
//! use guerrillamail_client::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), guerrillamail_client::Error> {
//!     let client = Client::new().await?;
//!     let email = client.create_email("myalias").await?;
//!     println!("Created: {}", email);
//!
//!     let messages = client.get_messages(&email).await?;
//!     for msg in messages {
//!         println!("From: {}, Subject: {}", msg.mail_from, msg.mail_subject);
//!     }
//!
//!     client.delete_email(&email).await?;
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod models;

pub use client::{Client, ClientBuilder};
pub use error::Error;
pub use models::{Attachment, EmailDetails, Message};

/// Result type alias for GuerrillaMail operations.
///
/// This is equivalent to `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

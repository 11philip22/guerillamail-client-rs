# guerrillamail-client

[![Crates.io](https://img.shields.io/crates/v/guerrillamail-client.svg)](https://crates.io/crates/guerrillamail-client)
[![Documentation](https://docs.rs/guerrillamail-client/badge.svg)](https://docs.rs/guerrillamail-client)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/11philip22/guerrillamail-client-rs/pulls)

![GuerrillaMail](https://img.guerrillamail.com/4/6/f/46f9fd8911b3a915c1fec119e9062d00.png)

An **async Rust API client** for the [GuerrillaMail](https://www.guerrillamail.com) temporary email service, built for testing and automation.

This crate lets you programmatically create disposable email addresses, poll inboxes, and fetch message contents using an idiomatic async Rust API built on `tokio` and `reqwest`.

> ⚠️ **Unofficial API**  
> GuerrillaMail does not provide a documented public API. This client reverse-engineers
> the web interface and may break if GuerrillaMail changes their frontend behavior.

## When to use this

- Email-based testing (signups, verification emails, password resets)
- Automation and scraping workflows
- CI / integration tests that need a disposable inbox
- Account generators and security tooling

## Features

- **Async/await first** — built on `tokio` and `reqwest`
- **Create temporary email addresses**
- **Poll inbox messages**
- **Fetch full email contents**
- **Forget/delete addresses**
- **Proxy support** (e.g. Burp, mitmproxy)
- **Configurable TLS + User-Agent**
- **Well-typed errors with proper chaining**

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
guerrillamail-client = "0.7.1"
tokio = { version = "1", features = ["full"] }
```

## Quick start

```rust
use guerrillamail_client::Client;

#[tokio::main]
async fn main() -> Result<(), guerrillamail_client::Error> {
    // Create a client (performs a bootstrap request)
    let client = Client::new().await?;

    // Create a temporary email address
    let email = client.create_email("myalias").await?;
    println!("Temporary email: {email}");

    // Poll inbox
    let messages = client.get_messages(&email).await?;
    for msg in messages {
        println!("From: {}", msg.mail_from);
        println!("Subject: {}", msg.mail_subject);
    }

    // Fetch full email body
    if let Some(msg) = messages.first() {
        let details = client.fetch_email(&email, &msg.mail_id).await?;
        println!("Body:\n{}", details.mail_body);
    }

    // Forget the email address
    client.delete_email(&email).await?;

    Ok(())
}
```

## Downloading attachments

```rust
use guerrillamail_client::Client;

#[tokio::main]
async fn main() -> Result<(), guerrillamail_client::Error> {
    let client = Client::new().await?;
    let email = client.create_email("myalias").await?;
    let messages = client.get_messages(&email).await?;

    if let Some(msg) = messages.first() {
        let attachments = client.list_attachments(&email, &msg.mail_id).await?;
        if let Some(attachment) = attachments.first() {
            let bytes = client
                .fetch_attachment(&email, &msg.mail_id, attachment)
                .await?;
            println!("Downloaded {} bytes", bytes.len());
        }
    }

    Ok(())
}
```

## Configuration via builder

For proxies, stricter TLS, custom user agents, or a different request timeout, use the builder API:

```rust
use guerrillamail_client::Client;

let client = Client::builder()
    .proxy("http://127.0.0.1:8080")
    .danger_accept_invalid_certs(false)
    .user_agent("my-app/1.0")
    .timeout(std::time::Duration::from_secs(30)) // default is 30s; customize as needed
    .build()
    .await?;
```

## Documentation

For detailed API documentation, visit [docs.rs/guerrillamail-client](https://docs.rs/guerrillamail-client).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgements

This project was inspired by and partially based on  
[GuerrillaMail-Python](https://github.com/rino-snow/GuerrillaMail-Python).

## Support

If this crate saves you time or helps your work, support is appreciated:

[![Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/11philip22)

## License

This project is licensed under the MIT License; see the [license](license) file for details.

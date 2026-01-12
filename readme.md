# guerrillamail-rs

![GuerrillaMail](https://img.guerrillamail.com/4/6/f/46f9fd8911b3a915c1fec119e9062d00.png)

An async Rust client for the [GuerrillaMail](https://www.guerrillamail.com) temporary email service.

## Features

- 🚀 **Async/await** - Built on tokio and reqwest
- 📧 **Create temporary emails** - Generate disposable email addresses
- 📬 **Check inbox** - Retrieve messages from your temporary inbox
- 🗑️ **Delete emails** - Clean up when done
- 🌐 **Proxy support** - Route requests through HTTP proxies

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
guerrillamail = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use guerrillamail::Client;

#[tokio::main]
async fn main() -> Result<(), guerrillamail::Error> {
    // Create a new client
    let client = Client::new().await?;
    
    // Create a temporary email address
    let email = client.create_email("myalias", None).await?;
    println!("Temporary email: {}", email);
    
    // Check for messages
    let messages = client.get_messages(&email).await?;
    for msg in messages {
        println!("From: {}", msg.mail_from);
        println!("Subject: {}", msg.mail_subject);
    }
    
    // Delete when done
    client.delete_email(&email).await?;
    
    Ok(())
}
```

## API

### `Client::new()`
Create a new GuerrillaMail client.

### `Client::with_proxy(proxy: Option<&str>)`
Create a client with an optional HTTP proxy.

```rust
let client = Client::with_proxy(Some("http://127.0.0.1:8080")).await?;
```

### `client.domains()`
Get the list of available email domains.

### `client.create_email(alias, domain)`
Create a temporary email. If `domain` is `None`, the server assigns one.

### `client.get_messages(email)`
Retrieve all messages for an email address.

### `client.delete_email(email)`
Delete/forget an email address.

## Message Fields

```rust
pub struct Message {
    pub mail_id: String,
    pub mail_from: String,
    pub mail_subject: String,
    pub mail_excerpt: String,
    pub mail_timestamp: String,
}
```

## License

MIT License - see [license](license) for details.

# telegram-notify

Tiny async Rust crate for sending Telegram bot messages to one
configured chat.

## Features

-   Minimal `send(msg)` API
-   Async (Tokio-based)
-   Environment-based configuration
-   Input validation (empty + max length)
-   Lightweight and dependency-minimal

---

## Installation

``` toml
[dependencies]
telegram-notify = { path = "../telegram-notify" }
tokio = { version = "1", features = ["rt", "macros"] }
```

---

## Environment

Set these environment variables before sending:

``` env
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

---

## Usage

``` rust
use telegram_notify::send;

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    send("trade executed").await?;
    Ok(())
}
```

---

## Getting your chat ID

Run the helper example:

``` bash
TELEGRAM_BOT_TOKEN=your_bot_token cargo run --example chat_id
```

Then send any message to your bot in Telegram.

The program will print:

``` text
chat_id = 123456789
```

Use that value as `TELEGRAM_CHAT_ID`.

---

## Example

``` bash
TELEGRAM_BOT_TOKEN=your_bot_token \
TELEGRAM_CHAT_ID=123456789 \
cargo run --example send
```

---

## Behavior

-   Messages are trimmed before sending
-   Empty messages are rejected
-   Messages longer than 4096 characters are rejected (Telegram limit)
-   Errors are returned to the caller

---

## Notes

-   You must manually start the bot in Telegram (`/start`) before
    sending messages
-   This crate sends plain text only
-   Designed for simple alerting / notification use cases

---

## License

MIT
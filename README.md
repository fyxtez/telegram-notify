# telegram-notify

Tiny async Rust crate for sending Telegram bot text and HTML messages to one configured chat.

## Features

- Minimal `send(msg)` API for plain text
- Generic `send_html(html)` API with Telegram HTML parse mode
- Link previews kept enabled above HTML text (supports the zero-width preview-link pattern)
- `escape_html(...)` helper for safely interpolating dynamic content
- Async (Tokio-based)
- Environment-based configuration
- Plain-text input validation
- Lightweight and dependency-minimal

---

## Installation

```toml
[dependencies]
telegram-notify = "1.1.0"
tokio = { version = "1", features = ["rt", "macros"] }
```

---

## Environment

Set these environment variables before sending:

```env
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

---

## Plain text

```rust
use telegram_notify::send;

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    send("trade executed").await?;
    Ok(())
}
```

---

## HTML

Use `escape_html` for any dynamic text inserted into markup:

```rust
use telegram_notify::{escape_html, send_html};

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    let text = escape_html("BTC < 100k & moving fast");
    let html = format!("<b>Alert</b>\n{text}");

    send_html(&html).await?;
    Ok(())
}
```

To select a media link preview without printing its URL, put a zero-width link first. `send_html` requests that Telegram show the preview above the text:

```rust
let html = format!(
    "<a href=\"{image_url}\">&#8205;</a><b>New post</b>\n{}",
    escape_html(post_text),
);

send_html(&html).await?;
```

Supported formatting is Telegram's HTML subset, including tags such as `<b>`, `<i>`, `<code>`, `<a>`, and `<blockquote>`.

---

## Examples

Plain text:

```bash
TELEGRAM_BOT_TOKEN=your_bot_token \
TELEGRAM_CHAT_ID=123456789 \
cargo run --example send
```

HTML + media preview + embedded block:

```bash
TELEGRAM_BOT_TOKEN=your_bot_token \
TELEGRAM_CHAT_ID=123456789 \
cargo run --example send_html
```

---

## Getting your chat ID

Run the helper example:

```bash
TELEGRAM_BOT_TOKEN=your_bot_token cargo run --example chat_id
```

Then send any message to your bot in Telegram. The program will print:

```text
chat_id = 123456789
```

Use that value as `TELEGRAM_CHAT_ID`.

---

## Behavior

- Plain messages are trimmed before sending
- Empty plain-text and HTML messages are rejected
- Plain messages longer than 4096 characters are rejected locally
- HTML length is validated by Telegram after entity parsing, because raw markup characters do not map 1:1 to Telegram's parsed 4096-character limit
- Telegram API errors are returned to the caller

---

## Notes

- You must manually start the bot in Telegram (`/start`) before sending messages
- `send_html` is deliberately generic; application-specific rendering stays in the caller

---

## License

MIT

# telegram-notify

Tiny async Rust crate for sending Telegram bot text and rich HTML messages to one configured chat.

## Features

- Minimal `send(msg)` API for plain text
- Generic `send_html(...)` API with Telegram HTML parse mode
- Backwards-compatible `send_html(&html)` calls
- Optional native photo/video attachment through `HtmlMessage`
- Optional generic inline URL button below an HTML message
- Link previews kept enabled above normal HTML text
- `escape_html(...)` helper for safely interpolating dynamic content
- Async (Tokio-based)
- Environment-based configuration

## Installation

```toml
[dependencies]
telegram-notify = "1.2.0"
tokio = { version = "1", features = ["rt", "macros"] }
```

## Environment

```env
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

## Plain text

```rust
use telegram_notify::send;

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    send("trade executed").await?;
    Ok(())
}
```

## HTML

Existing HTML calls continue to work:

```rust
use telegram_notify::{escape_html, send_html};

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    let text = escape_html("BTC < 100k & moving fast");
    send_html(format!("<b>Alert</b>\n{text}")).await?;
    Ok(())
}
```

For image-only notifications, a zero-width first link can still select Telegram's normal link preview:

```rust
let html = format!(
    "<a href=\"{image_url}\">&#8205;</a><b>New post</b>\n{}",
    escape_html(post_text),
);

send_html(html).await?;
```

## HTML with native media and a button

Use `HtmlMessage` when a direct media URL should be sent as Telegram media instead of relying on a link preview, or when the message needs an inline URL button:

```rust
use telegram_notify::{send_html, HtmlMessage};

let message = HtmlMessage::new("<b>New event</b>\nVideo attached")
    .video("https://example.com/video.mp4")
    .button("Open source ↗", "https://example.com/event");

send_html(message).await?;
```

`photo(...)` is also available. Media captions are subject to Telegram's media-caption limit, so callers should keep HTML with native media compact.

Supported formatting is Telegram's HTML subset, including `<b>`, `<i>`, `<code>`, `<a>`, and `<blockquote>`.

## Examples

```bash
TELEGRAM_BOT_TOKEN=your_bot_token \
TELEGRAM_CHAT_ID=123456789 \
cargo run --example send_html
```

To additionally test native video delivery:

```bash
TELEGRAM_TEST_VIDEO_URL='https://example.com/video.mp4' \
TELEGRAM_BOT_TOKEN=your_bot_token \
TELEGRAM_CHAT_ID=123456789 \
cargo run --example send_html
```

## Getting your chat ID

```bash
TELEGRAM_BOT_TOKEN=your_bot_token cargo run --example chat_id
```

## Behavior

- Empty plain-text and HTML messages are rejected
- Plain messages longer than 4096 characters are rejected locally
- HTML length is validated by Telegram after entity parsing
- Native media URLs and inline-button URLs are validated before sending
- Telegram API errors are returned to the caller
- `send_html` remains deliberately generic; application-specific rendering stays in the caller

## License

MIT

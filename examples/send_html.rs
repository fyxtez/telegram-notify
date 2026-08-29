use telegram_notify::{escape_html, send_html};

/// Sends a formatted HTML message with a media preview above the text.
///
/// Run with:
/// TELEGRAM_BOT_TOKEN=... TELEGRAM_CHAT_ID=... cargo run --example send_html
#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    let preview_url = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    let dynamic_text = escape_html("HTML works: BTC < 100k & still moving.");

    // The zero-width first link selects the preview without printing a raw URL
    // in the visible message body. send_html keeps that preview above the text.
    let html = format!(
        "<a href=\"{preview_url}\">&#8205;</a><b>telegram-notify HTML test</b>\n\n{dynamic_text}\n\n<blockquote><b>Embedded block</b>\nThis is how nested content can be rendered.</blockquote>"
    );

    send_html(&html).await?;
    Ok(())
}

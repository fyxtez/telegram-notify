use telegram_notify::{escape_html, send_html, HtmlMessage};

/// Sends formatted HTML with an image preview and an inline URL button.
///
/// To also test native video delivery, set TELEGRAM_TEST_VIDEO_URL to a public
/// direct MP4 URL before running the example.
#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    let preview_url = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    let dynamic_text = escape_html("HTML works: BTC < 100k & still moving.");
    let html = format!(
        "<a href=\"{preview_url}\">&#8205;</a><b>telegram-notify HTML test</b>\n\n{dynamic_text}\n\n<blockquote><b>Embedded block</b>\nRich HTML can include Telegram media and buttons.</blockquote>"
    );

    let mut message = HtmlMessage::new(html).button("Open Rust ↗", "https://www.rust-lang.org/");

    if let Ok(video_url) = std::env::var("TELEGRAM_TEST_VIDEO_URL") {
        message = message.video(video_url);
    }

    send_html(message).await?;
    Ok(())
}

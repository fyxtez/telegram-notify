//! Tiny async Telegram notification crate.
//!
//! This crate provides minimal `send` and generic `send_html` APIs for sending
//! bot notifications to one configured chat.
//!
//! # Environment variables
//!
//! - `TELEGRAM_BOT_TOKEN`
//! - `TELEGRAM_CHAT_ID`

use std::env;
use std::sync::OnceLock;
use teloxide::prelude::*;
use teloxide::types::{
    ChatId, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, LinkPreviewOptions, ParseMode,
};
use url::Url;

/// Maximum Telegram plain-text message length.
const MAX_MESSAGE_LEN: usize = 4096;

static BOT: OnceLock<Bot> = OnceLock::new();

/// Optional media attached to a [`HtmlMessage`].
///
/// Images can still use Telegram's normal link preview mechanism. `Video` is
/// useful when a direct MP4 URL would otherwise render as a plain link instead
/// of visual media in a normal `sendMessage` preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlMedia {
    Photo(String),
    Video(String),
}

/// Generic HTML message options for [`send_html`].
///
/// `send_html("<b>Hello</b>")` remains valid through the `From<&str>` impl,
/// while callers that need Telegram-native media or an inline URL button can
/// build an `HtmlMessage` explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlMessage {
    html: String,
    media: Option<HtmlMedia>,
    button: Option<(String, String)>,
}

impl HtmlMessage {
    pub fn new(html: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            media: None,
            button: None,
        }
    }

    pub fn photo(mut self, url: impl Into<String>) -> Self {
        self.media = Some(HtmlMedia::Photo(url.into()));
        self
    }

    pub fn video(mut self, url: impl Into<String>) -> Self {
        self.media = Some(HtmlMedia::Video(url.into()));
        self
    }

    /// Adds one generic Telegram URL button below the message.
    pub fn button(mut self, text: impl Into<String>, url: impl Into<String>) -> Self {
        self.button = Some((text.into(), url.into()));
        self
    }
}

impl From<&str> for HtmlMessage {
    fn from(html: &str) -> Self {
        Self::new(html)
    }
}

impl From<&String> for HtmlMessage {
    fn from(html: &String) -> Self {
        Self::new(html.clone())
    }
}

impl From<String> for HtmlMessage {
    fn from(html: String) -> Self {
        Self::new(html)
    }
}

/// Errors returned by this crate.
#[derive(Debug)]
pub enum NotifyError {
    /// Required environment variable is missing.
    MissingEnv(&'static str),
    /// `TELEGRAM_CHAT_ID` could not be parsed as `i64`.
    InvalidChatId,
    /// A media/button URL is invalid.
    InvalidUrl(String),
    /// Message is empty after trimming whitespace.
    EmptyMessage,
    /// Plain-text message exceeds Telegram's 4096 character limit.
    MessageTooLong,
    /// Telegram API request failed.
    Telegram(teloxide::RequestError),
}

impl std::fmt::Display for NotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnv(name) => write!(f, "missing environment variable: {name}"),
            Self::InvalidChatId => write!(f, "invalid TELEGRAM_CHAT_ID"),
            Self::InvalidUrl(url) => write!(f, "invalid URL: {url}"),
            Self::EmptyMessage => write!(f, "message is empty"),
            Self::MessageTooLong => write!(f, "message exceeds 4096 characters"),
            Self::Telegram(err) => write!(f, "telegram request failed: {err}"),
        }
    }
}

impl std::error::Error for NotifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Telegram(err) => Some(err),
            _ => None,
        }
    }
}

impl From<teloxide::RequestError> for NotifyError {
    fn from(err: teloxide::RequestError) -> Self {
        Self::Telegram(err)
    }
}

fn validated_non_empty(msg: &str) -> Result<&str, NotifyError> {
    let msg = msg.trim();

    if msg.is_empty() {
        return Err(NotifyError::EmptyMessage);
    }

    Ok(msg)
}

fn validated_message(msg: &str) -> Result<&str, NotifyError> {
    let msg = validated_non_empty(msg)?;

    if msg.chars().count() > MAX_MESSAGE_LEN {
        return Err(NotifyError::MessageTooLong);
    }

    Ok(msg)
}

fn parse_url(url: &str) -> Result<Url, NotifyError> {
    Url::parse(url).map_err(|_| NotifyError::InvalidUrl(url.to_string()))
}

fn load_bot_token() -> Result<String, NotifyError> {
    env::var("TELEGRAM_BOT_TOKEN").map_err(|_| NotifyError::MissingEnv("TELEGRAM_BOT_TOKEN"))
}

fn load_chat_id() -> Result<ChatId, NotifyError> {
    let chat_id = env::var("TELEGRAM_CHAT_ID")
        .map_err(|_| NotifyError::MissingEnv("TELEGRAM_CHAT_ID"))?
        .parse::<i64>()
        .map_err(|_| NotifyError::InvalidChatId)?;

    Ok(ChatId(chat_id))
}

fn bot() -> Result<&'static Bot, NotifyError> {
    if let Some(bot) = BOT.get() {
        return Ok(bot);
    }

    let token = load_bot_token()?;
    Ok(BOT.get_or_init(|| Bot::new(token)))
}

fn inline_keyboard(button: Option<&(String, String)>) -> Result<Option<InlineKeyboardMarkup>, NotifyError> {
    let Some((text, url)) = button else {
        return Ok(None);
    };

    // FEATURE: URL buttons are kept generic in telegram-notify so applications
    // can add actions such as "View Tweet" without coupling this crate to X.
    let button = InlineKeyboardButton::url(text.clone(), parse_url(url)?);
    Ok(Some(InlineKeyboardMarkup::new([[button]])))
}

/// Escapes dynamic text so it can be safely inserted into Telegram HTML.
pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

/// Sends a plain text Telegram message to the configured chat.
pub async fn send(msg: &str) -> Result<(), NotifyError> {
    let msg = validated_message(msg)?;
    let bot = bot()?;
    let chat_id = load_chat_id()?;

    bot.send_message(chat_id, msg).await?;
    Ok(())
}

/// Sends a generic Telegram HTML message to the configured chat.
///
/// A plain string keeps the original `send_html(&html)` behavior. Build an
/// [`HtmlMessage`] when you also need a native photo/video or one inline URL
/// button below the message.
///
/// Without native media, link previews remain enabled above the text so the
/// zero-width first-link pattern continues to work for image previews.
pub async fn send_html(message: impl Into<HtmlMessage>) -> Result<(), NotifyError> {
    let message = message.into();
    let html = validated_non_empty(&message.html)?.to_string();
    let bot = bot()?;
    let chat_id = load_chat_id()?;
    let keyboard = inline_keyboard(message.button.as_ref())?;

    match message.media {
        Some(HtmlMedia::Photo(url)) => {
            let mut request = bot
                .send_photo(chat_id, InputFile::url(parse_url(&url)?))
                .caption(html)
                .parse_mode(ParseMode::Html)
                .show_caption_above_media(true);

            if let Some(keyboard) = keyboard {
                request = request.reply_markup(keyboard);
            }

            request.await?;
        }
        Some(HtmlMedia::Video(url)) => {
            let mut request = bot
                .send_video(chat_id, InputFile::url(parse_url(&url)?))
                .caption(html)
                .parse_mode(ParseMode::Html)
                .show_caption_above_media(true)
                .supports_streaming(true);

            if let Some(keyboard) = keyboard {
                request = request.reply_markup(keyboard);
            }

            request.await?;
        }
        None => {
            let link_preview = LinkPreviewOptions {
                is_disabled: false,
                url: None,
                prefer_small_media: false,
                prefer_large_media: true,
                show_above_text: true,
            };

            let mut request = bot
                .send_message(chat_id, html)
                .parse_mode(ParseMode::Html)
                .link_preview_options(link_preview);

            if let Some(keyboard) = keyboard {
                request = request.reply_markup(keyboard);
            }

            request.await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn display_messages_exist() {
        assert_eq!(NotifyError::MissingEnv("X").to_string(), "missing environment variable: X");
        assert_eq!(NotifyError::InvalidChatId.to_string(), "invalid TELEGRAM_CHAT_ID");
        assert_eq!(NotifyError::EmptyMessage.to_string(), "message is empty");
        assert_eq!(NotifyError::MessageTooLong.to_string(), "message exceeds 4096 characters");
        assert_eq!(NotifyError::InvalidUrl("bad".into()).to_string(), "invalid URL: bad");
    }

    #[test]
    fn html_message_builder_preserves_generic_options() {
        let message = HtmlMessage::new("<b>hello</b>")
            .video("https://example.com/test.mp4")
            .button("Open", "https://example.com");

        assert_eq!(message.html, "<b>hello</b>");
        assert_eq!(message.media, Some(HtmlMedia::Video("https://example.com/test.mp4".into())));
        assert_eq!(message.button, Some(("Open".into(), "https://example.com".into())));
    }

    #[test]
    fn invalid_button_url_is_rejected_before_request() {
        let button = ("Open".to_string(), "not a url".to_string());
        assert!(matches!(inline_keyboard(Some(&button)), Err(NotifyError::InvalidUrl(_))));
    }

    #[test]
    fn validated_message_rejects_empty() {
        let err = validated_message("   ").unwrap_err();
        assert!(matches!(err, NotifyError::EmptyMessage));
    }

    #[test]
    fn validated_message_rejects_too_long() {
        let msg = "a".repeat(4097);
        let err = validated_message(&msg).unwrap_err();
        assert!(matches!(err, NotifyError::MessageTooLong));
    }

    #[test]
    fn validated_message_trims_ok() {
        let msg = validated_message("  hello  ").unwrap();
        assert_eq!(msg, "hello");
    }

    #[test]
    fn html_validation_only_rejects_empty_content() {
        assert_eq!(validated_non_empty("  <b>hello</b>  ").unwrap(), "<b>hello</b>");
        assert!(matches!(validated_non_empty(" \n\t ").unwrap_err(), NotifyError::EmptyMessage));
    }

    #[test]
    fn escape_html_protects_telegram_markup() {
        assert_eq!(escape_html("BTC < 100k & \"moving\" > fast"), "BTC &lt; 100k &amp; &quot;moving&quot; &gt; fast");
    }

    #[test]
    fn load_bot_token_missing() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::remove_var("TELEGRAM_BOT_TOKEN"); }
        let err = load_bot_token().unwrap_err();
        assert!(matches!(err, NotifyError::MissingEnv("TELEGRAM_BOT_TOKEN")));
    }

    #[test]
    fn load_bot_token_ok() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::set_var("TELEGRAM_BOT_TOKEN", "test_token"); }
        let token = load_bot_token().unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn load_chat_id_missing() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::remove_var("TELEGRAM_CHAT_ID"); }
        let err = load_chat_id().unwrap_err();
        assert!(matches!(err, NotifyError::MissingEnv("TELEGRAM_CHAT_ID")));
    }

    #[test]
    fn load_chat_id_invalid() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::set_var("TELEGRAM_CHAT_ID", "not_a_number"); }
        let err = load_chat_id().unwrap_err();
        assert!(matches!(err, NotifyError::InvalidChatId));
    }

    #[test]
    fn load_chat_id_ok() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::set_var("TELEGRAM_CHAT_ID", "123"); }
        let chat_id = load_chat_id().unwrap();
        assert_eq!(chat_id, ChatId(123));
    }
}

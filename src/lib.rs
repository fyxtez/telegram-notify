//! Tiny async Telegram notification crate.
//!
//! This crate provides a minimal `send` API for sending Telegram bot messages
//! to a single configured chat.
//!
//! # Environment variables
//!
//! - `TELEGRAM_BOT_TOKEN`
//! - `TELEGRAM_CHAT_ID`
//!
//! # Example
//!
//! ```no_run
//! use telegram_notify::send;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), telegram_notify::NotifyError> {
//!     send("trade executed").await?;
//!     Ok(())
//! }
//! ```

use once_cell::sync::OnceCell;
use std::env;
use teloxide::prelude::*;
use teloxide::types::ChatId;

/// Maximum Telegram text message length.
const MAX_MESSAGE_LEN: usize = 4096;

static CLIENT: OnceCell<(Bot, ChatId)> = OnceCell::new();

/// Errors returned by this crate.
#[derive(Debug)]
pub enum NotifyError {
    /// Required environment variable is missing.
    MissingEnv(&'static str),
    /// `TELEGRAM_CHAT_ID` could not be parsed as `i64`.
    InvalidChatId,
    /// Message is empty after trimming whitespace.
    EmptyMessage,
    /// Message exceeds Telegram's 4096 character limit.
    MessageTooLong,
    /// Telegram API request failed.
    Telegram(teloxide::RequestError),
}

impl std::fmt::Display for NotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnv(name) => write!(f, "missing environment variable: {name}"),
            Self::InvalidChatId => write!(f, "invalid TELEGRAM_CHAT_ID"),
            Self::EmptyMessage => write!(f, "message is empty"),
            Self::MessageTooLong => write!(f, "message exceeds 4096 characters"),
            Self::Telegram(err) => write!(f, "telegram request failed: {err}"),
        }
    }
}

impl std::error::Error for NotifyError {}

impl From<teloxide::RequestError> for NotifyError {
    fn from(err: teloxide::RequestError) -> Self {
        Self::Telegram(err)
    }
}

fn validated_message(msg: &str) -> Result<&str, NotifyError> {
    let msg = msg.trim();

    if msg.is_empty() {
        return Err(NotifyError::EmptyMessage);
    }

    if msg.chars().count() > MAX_MESSAGE_LEN {
        return Err(NotifyError::MessageTooLong);
    }

    Ok(msg)
}

fn load_config() -> Result<(String, ChatId), NotifyError> {
    let token =
        env::var("TELEGRAM_BOT_TOKEN").map_err(|_| NotifyError::MissingEnv("TELEGRAM_BOT_TOKEN"))?;

    let chat_id = env::var("TELEGRAM_CHAT_ID")
        .map_err(|_| NotifyError::MissingEnv("TELEGRAM_CHAT_ID"))?
        .parse::<i64>()
        .map_err(|_| NotifyError::InvalidChatId)?;

    Ok((token, ChatId(chat_id)))
}

fn client() -> Result<&'static (Bot, ChatId), NotifyError> {
    CLIENT.get_or_try_init(|| {
        let (token, chat_id) = load_config()?;
        Ok((Bot::new(token), chat_id))
    })
}

/// Sends a plain text Telegram message to the configured chat.
///
/// The target chat is taken from the `TELEGRAM_CHAT_ID` environment variable,
/// and the bot token is taken from `TELEGRAM_BOT_TOKEN`.
///
/// # Errors
///
/// Returns an error if:
///
/// - required environment variables are missing
/// - `TELEGRAM_CHAT_ID` is invalid
/// - the message is empty after trimming
/// - the message exceeds Telegram's 4096 character limit
/// - Telegram rejects the request
pub async fn send(msg: &str) -> Result<(), NotifyError> {
    let msg = validated_message(msg)?;

    let (bot, chat_id) = client()?;
    bot.send_message(*chat_id, msg).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn display_messages_exist() {
        assert_eq!(
            NotifyError::MissingEnv("X").to_string(),
            "missing environment variable: X"
        );
        assert_eq!(
            NotifyError::InvalidChatId.to_string(),
            "invalid TELEGRAM_CHAT_ID"
        );
        assert_eq!(NotifyError::EmptyMessage.to_string(), "message is empty");
        assert_eq!(
            NotifyError::MessageTooLong.to_string(),
            "message exceeds 4096 characters"
        );
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
    fn load_config_missing_token() {
        let _guard = ENV_LOCK.lock().unwrap();

        unsafe {
            env::remove_var("TELEGRAM_BOT_TOKEN");
            env::remove_var("TELEGRAM_CHAT_ID");
            env::set_var("TELEGRAM_CHAT_ID", "123");
        }

        let err = load_config().unwrap_err();
        assert!(matches!(err, NotifyError::MissingEnv("TELEGRAM_BOT_TOKEN")));
    }

    #[test]
    fn load_config_missing_chat_id() {
        let _guard = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
            env::remove_var("TELEGRAM_CHAT_ID");
        }

        let err = load_config().unwrap_err();
        assert!(matches!(err, NotifyError::MissingEnv("TELEGRAM_CHAT_ID")));
    }

    #[test]
    fn load_config_invalid_chat_id() {
        let _guard = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
            env::set_var("TELEGRAM_CHAT_ID", "not_a_number");
        }

        let err = load_config().unwrap_err();
        assert!(matches!(err, NotifyError::InvalidChatId));
    }

    #[test]
    fn load_config_ok() {
        let _guard = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
            env::set_var("TELEGRAM_CHAT_ID", "123");
        }

        let (token, chat_id) = load_config().unwrap();
        assert_eq!(token, "test_token");
        assert_eq!(chat_id, ChatId(123));
    }

}
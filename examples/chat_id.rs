use teloxide::prelude::*;

/// Prints the chat ID for incoming messages sent to the bot.
///
/// Run this example, then send any message to your bot in Telegram.
/// The program will print the chat ID you should store as `TELEGRAM_CHAT_ID`.
#[tokio::main]
async fn main() {
    let bot = Bot::new(std::env::var("TELEGRAM_BOT_TOKEN").expect("missing TELEGRAM_BOT_TOKEN"));

    teloxide::repl(bot, |msg: Message| async move {
        println!("chat_id = {}", msg.chat.id.0);
        respond(())
    })
    .await;
}

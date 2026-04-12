use telegram_notify::send;

#[tokio::main]
async fn main() -> Result<(), telegram_notify::NotifyError> {
    send("trade executed").await?;
    Ok(())
}

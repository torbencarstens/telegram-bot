use std::{env, fs};
use teloxide::{prelude::*, utils::command::BotCommand};
use tokio;
use std::error::Error;
use tempdir::TempDir;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "help")]
    Help,
    #[command(description = "inform the user who has the hand on the mouse")]
    WerHatDieHandAnDerMaus,
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::WerHatDieHandAnDerMaus => {
            cx.answer(format!("Tim")).await?
        }
        command_name => {
            unimplemented!("{:#?}", command_name)
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    teloxide::enable_logging!();
    let bot_name = env::var("BOT_NAME")?;
    log::info!("Starting {}...", bot_name);

    let bot = Bot::from_env().auto_send();

    teloxide::commands_repl(bot, bot_name, answer).await;

    Ok(())
}

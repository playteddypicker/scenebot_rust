use dotenv::dotenv;
use env_logger::init;
use log::error;
use serenity::prelude::*;
use std::env;

mod event_handler;
mod events;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init();

    let token = if env::var("DEV").unwrap_or("0".to_string()) == "1" {
        env::var("DEV_TOKEN").expect("couldn't find token.")
    } else {
        env::var("DISCORD_TOKEN").expect("couldn't find token.")
    };

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(event_handler::DiscordEventHandler)
        .await
        .expect("Err creating client.");

    if let Err(why) = client.start().await {
        error!("Err creating client: {:?}", why);
    }
}

use dotenv::dotenv;
use serenity::prelude::*;
use std::env;

mod events;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("couldn't find token.");
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&token, intents)
        .event_handler(events::DiscordEventHandler)
        .await
        .expect("Err creating client.");

    if let Err(why) = client.start().await {
        println!("Err creating client: {:?}", why);
    }
}

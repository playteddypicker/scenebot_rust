use dotenv::dotenv;
use env_logger::init;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client as DBClient,
};
use serenity::prelude::*;

mod command_handler;
mod event_handler;
mod events;
mod utils;

use std::{collections::HashMap, env, error::Error, num::NonZeroU64, sync::Arc};
use tokio::sync::{Mutex, RwLock};

struct GlobalGuildConfigs;
impl TypeMapKey for GlobalGuildConfigs {
    type Value =
        Arc<RwLock<HashMap<NonZeroU64, Arc<Mutex<crate::utils::guild_config::GuildConfig>>>>>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    init();

    let token = if env::var("DEV").unwrap_or("0".to_string()) == "1" {
        env::var("DEV_TOKEN").expect("couldn't find token.")
    } else {
        env::var("DISCORD_TOKEN").expect("couldn't find token.")
    };

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client_uri = env::var("DB_URI").expect("couldn't' find db uri");
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;

    let db_client = DBClient::with_options(options)?;

    let handler = event_handler::DiscordEventHandler {
        database: db_client,
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client.");

    {
        let mut data = client.data.write().await;
        data.insert::<GlobalGuildConfigs>(Arc::new(RwLock::new(HashMap::default())));
    }

    client.start().await?;
    Ok(())
}

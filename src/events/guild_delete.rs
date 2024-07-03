use crate::utils::guild_config::GuildConfig;
use serenity::{all::UnavailableGuild, client::Context, model::guild::Guild};

use log::{error, info, warn};

pub async fn db_delete(
    ctx: &Context,
    incomplete: &UnavailableGuild,
    _: &Option<Guild>,
    database: &mongodb::Client,
) {
    let guildid = incomplete.id.get();
    let collections: mongodb::Collection<bson::document::Document> = database
        .database("scene")
        .collection(std::env::var("BOT_DB_NAME").unwrap().as_str());

    if let Err(why) = collections
        .find_one_and_delete(mongodb::bson::doc! { "guild_id" : guildid as f64 }, None)
        .await
    {
        error!(
            "Couldn't delete new DB from: guildid: {} {:?}",
            guildid, why
        );
    }

    match GuildConfig::delete(&incomplete.id, ctx).await {
        Ok(_) => info!("guild config on memory-hashmap has been deleted completely."),
        Err(_) => warn!("guild config on memory-hashmap doesn't exist."),
    };
}

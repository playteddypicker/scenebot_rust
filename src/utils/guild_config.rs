use crate::{utils::scene_core::ImageSize, GlobalGuildConfigs};
use log::{error, info};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serenity::{
    all::Permissions, builder::CreateCommand, client::Context, gateway::ActivityData,
    model::id::GuildId,
};
use std::num::NonZeroU64;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: NonZeroU64,
    pub auto_magnitute_enable: bool,
    pub auto_magnitute_config: ImageSize,
    pub auto_transfer_webp: bool,
}

impl GuildConfig {
    pub fn new(guild: &GuildId) -> Self {
        Self {
            guild_id: NonZeroU64::new(guild.get()).unwrap(),
            auto_magnitute_enable: false,
            auto_magnitute_config: ImageSize::Auto,
            auto_transfer_webp: false,
        }
    }

    pub fn load(
        guild: &GuildId,
        auto_magnitute_enable_input: bool,
        auto_magnitute_config_input: ImageSize,
        auto_transfer_webp_input: bool,
    ) -> Self {
        Self {
            guild_id: NonZeroU64::new(guild.get()).unwrap(),
            auto_magnitute_enable: auto_magnitute_enable_input,
            auto_magnitute_config: auto_magnitute_config_input,
            auto_transfer_webp: auto_transfer_webp_input,
        }
    }

    pub async fn delete(guild: &GuildId, ctx: &Context) -> Result<(), ()> {
        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<GlobalGuildConfigs>()
                .expect("poisened")
                .clone()
        };
        let mut guilds_config = counter_lock.write().await;
        match guilds_config.remove(&NonZeroU64::new(guild.get()).unwrap()) {
            Some(_) => Ok(()),
            None => Err(()),
        }
    }

    pub async fn boot(ctx: &Context, database: &mongodb::Client) {
        info!(
            "loaded {} guilds. fetching config data from DB..",
            ctx.cache.guilds().len()
        );

        let collections: Arc<Mutex<mongodb::Collection<bson::document::Document>>> =
            Arc::new(Mutex::new(
                database
                    .database("scene")
                    .collection(std::env::var("BOT_DB_NAME").unwrap().as_str()),
            ));

        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<GlobalGuildConfigs>()
                .expect("poisened")
                .clone()
        };
        ctx.set_activity(Some(ActivityData::playing("부팅")));

        for guild in ctx.cache.guilds() {
            let sync_collections = collections.clone();
            let sync_guilds_config = counter_lock.clone();
            if let Err(why) = guild
                .create_command(
                    &ctx.http,
                    CreateCommand::new("update")
                        .description("봇의 업데이트를 확인해요")
                        .default_member_permissions(Permissions::ADMINISTRATOR),
                )
                .await
            {
                error!(
                    "error occured while creating update command at {}\n{:#?}",
                    guild, why
                );
            }
            tokio::spawn(async move {
                info!("loading guild {}..", NonZeroU64::new(guild.get()).unwrap());

                let collections = sync_collections.lock().await;
                let mut guilds_config = sync_guilds_config.write().await;
                let find_result = collections
                    .find_one(
                        doc! {
                            "guild_id" : guild.get() as f64
                        },
                        None,
                    )
                    .await;
                match find_result {
                    Ok(x) => {
                        let new_config = match x {
                            Some(document) => GuildConfig::load(
                                &guild,
                                document.get_bool("auto_magnitute_enable").unwrap_or(false),
                                ImageSize::string_to_value(
                                    document.get_str("auto_magnitute_config").unwrap_or("Auto"),
                                ),
                                document.get_bool("auto_transfer_webp").unwrap_or(false),
                            ),
                            None => {
                                if let Err(why) = collections
                                    .insert_one(
                                        doc! {
                                                "guild_id" : guild.get() as f64,
                                                "auto_magnitute_enable" : false,
                                                "auto_magnitute_config" : "Auto",
                                                "auto_transfer_webp": false,
                                        },
                                        None,
                                    )
                                    .await
                                {
                                    error!(
                                        "Couldn't added new DB to: guildid: {}, {:?}",
                                        guild.get(),
                                        why
                                    );
                                    return;
                                }

                                GuildConfig::new(&guild)
                            }
                        };

                        guilds_config.insert(
                            NonZeroU64::new(guild.get()).unwrap(),
                            std::sync::Arc::new(tokio::sync::Mutex::new(new_config)),
                        );
                    }
                    Err(why) => {
                        error!(
                            "an error occured when loading data from DB: guildid: {}\n{:?}",
                            guild.get(),
                            why
                        );
                    }
                }
                info!("guild {} loaded.", guild.get());
            });
        }

        ctx.set_activity(Some(ActivityData::playing(format!(
            "이모지 확대용 봇 | {}개의 서버에서 일하는중",
            ctx.cache.guilds().len()
        ))));

        info!("booting complete.");
    }
}

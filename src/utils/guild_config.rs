use crate::{utils::scene_core::ImageSize, GlobalGuildConfigs};
use log::{error, info};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serenity::{
    builder::CreateCommand,
    client::Context,
    gateway::ActivityData,
    model::{id::GuildId, Permissions},
};
use std::num::NonZeroU64;

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: NonZeroU64,
    pub auto_magnitute_enable: bool,
    pub auto_magnitute_config: ImageSize,
}

impl GuildConfig {
    pub fn new(guild: &GuildId) -> Self {
        Self {
            guild_id: guild.0,
            auto_magnitute_enable: false,
            auto_magnitute_config: ImageSize::Auto,
        }
    }

    pub fn load(
        guild: &GuildId,
        auto_magnitute_enable_input: bool,
        auto_magnitute_config_input: ImageSize,
    ) -> Self {
        Self {
            guild_id: guild.0,
            auto_magnitute_enable: auto_magnitute_enable_input,
            auto_magnitute_config: auto_magnitute_config_input,
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
        match guilds_config.remove(&guild.0) {
            Some(_) => Ok(()),
            None => Err(()),
        }
    }

    pub async fn boot(ctx: &Context, database: &mongodb::Client) {
        info!(
            "loaded {} guilds. fetching config data from DB..",
            ctx.cache.guilds().len()
        );

        //asynchronous하게 처리하게 바꾸기. 지금은 공부를 덜해서 못바꾸겠음 ㅅㅂ
        let collections: mongodb::Collection<bson::document::Document> = database
            .database("scene")
            .collection(std::env::var("BOT_DB_NAME").unwrap().as_str());

        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<GlobalGuildConfigs>()
                .expect("poisened")
                .clone()
        };
        let mut guilds_config = counter_lock.write().await;
        ctx.set_activity(Some(ActivityData::playing("부팅")));

        for guild in ctx.cache.guilds() {
            let (_, find_result, _) = tokio::join!(
                guild.create_application_command(
                    &ctx.http,
                    CreateCommand::new("update")
                        .description("봇의 업데이트를 확인해요")
                        .default_member_permissions(Permissions::ADMINISTRATOR),
                ),
                collections.find_one(
                    doc! {
                        "guild_id" : guild.0.get() as f64
                    },
                    None,
                ),
                crate::command_handler::explicit_command_list::COMMAND_LIST
                    .register_commands(guild, ctx)
            );

            match find_result {
                Ok(x) => {
                    let new_config = match x {
                        Some(document) => GuildConfig::load(
                            &guild,
                            document.get_bool("auto_magnitute_enable").unwrap_or(false),
                            ImageSize::string_to_value(
                                document.get_str("auto_magnitute_config").unwrap_or("Auto"),
                            ),
                        ),
                        None => {
                            if let Err(why) = collections
                                .insert_one(
                                    doc! {
                                            "guild_id" : guild.0.get() as f64,
                                            "auto_magnitute_enable" : false,
                                            "auto_magnitute_config" : "Auto",
                                    },
                                    None,
                                )
                                .await
                            {
                                error!("Couldn't added new DB to: guildid: {}, {:?}", guild.0, why);
                                return;
                            }

                            GuildConfig::new(&guild)
                        }
                    };

                    guilds_config.insert(
                        guild.0,
                        std::sync::Arc::new(tokio::sync::Mutex::new(new_config)),
                    );
                }
                Err(why) => {
                    error!(
                        "an error occured when loading data from DB: guildid: {}\n{:?}",
                        guild.0, why
                    );
                }
            }
        }
        ctx.set_activity(Some(ActivityData::playing(format!(
            "이모지 확대용 봇 | {}개의 서버에서 일하는중",
            ctx.cache.guilds().len()
        ))));

        info!("booting complete.");
    }
}

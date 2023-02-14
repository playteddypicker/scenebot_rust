use serenity::{
    builder::CreateCommand,
    client::Context,
    gateway::ActivityData,
    model::{guild::Guild, id::UserId, prelude::Permissions},
};

use log::{error, info};
use mongodb::bson::doc;

use std::num::NonZeroU64;

use crate::utils::guild_config::GuildConfig;
use crate::GlobalGuildConfigs;

pub async fn senddm(ctx: &Context, guild: &Guild, is_new: Option<bool>) {
    if is_new.is_none() || !is_new.unwrap() {
        return;
    }
    let teddypicker = UserId(NonZeroU64::new(653157614452211712).unwrap());
    match teddypicker.create_dm_channel(&ctx.http).await {
        Ok(channel) => {
            channel
                .say(
                    &ctx.http,
                    format!(
                        "봇이 **{}**에 추가됨, 서버 수 : {}개",
                        guild.name,
                        ctx.cache.guilds().len()
                    ),
                )
                .await
                .expect("Error occured while sending dm to teddypicker.");
        }
        Err(why) => error!("error occured while creating dm channel: {:?}", why),
    }
}

pub async fn new_guild_added(ctx: &Context, guild: &Guild, is_new: Option<bool>) {
    if is_new.is_none() || !is_new.unwrap() {
        return;
    }

    info!("new guild added: {}, ID: {}", guild.name, guild.id);

    if let Err(why) = guild
        .id
        .create_application_command(
            &ctx.http,
            CreateCommand::new("update")
                .description("봇의 업데이트를 확인해요")
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await
    {
        error!(
            "error occured while creating update command at {}\n{:#?}",
            guild.id, why
        );
    }
}

//DB Fetch
//먼저 DB에 기존 서버 데이터가 있는지 검사
//있으면 Fetch, 없으면 Default값으로 새로 저장하고 메모리에 띄움
pub async fn db_fetch(
    ctx: &Context,
    guild: &Guild,
    is_new: Option<bool>,
    database: &mongodb::Client,
) {
    if is_new.is_none() || !is_new.unwrap() {
        return;
    }

    let collections = database.database("scene").collection("scene_guilds");
    let guildid = guild.id.0.get();
    let _ = collections
        .find_one_and_delete(doc! { "guild_id" : guildid as f64 }, None)
        .await;
    let new_config = GuildConfig::new(&guild.id);

    if let Err(why) = collections
        .insert_one(
            doc! {
                    "guild_id" : guildid as f64,
                    "auto_magnitute_enable" : false,
                    "auto_magnitute_config" : "Auto",
            },
            None,
        )
        .await
    {
        error!(
            "Couldn't added new DB to: guildid: {}, name: {}, {:?}",
            guild.id.0, guild.name, why
        );
        return;
    }

    info!(
        "new DB added to : guildid: {}, name: {}",
        guild.id.0, guild.name
    );

    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GlobalGuildConfigs>()
            .expect("poisened")
            .clone()
    };
    let mut guilds_config = counter_lock.write().await;
    guilds_config
        .entry(guild.id.0)
        .or_insert(std::sync::Arc::new(tokio::sync::Mutex::new(new_config)));
}

pub async fn set_bot_status(ctx: &Context) {
    ctx.set_activity(Some(ActivityData::playing(format!(
        "이모지 확대용 봇 | {}개의 서버에서 일하는중",
        ctx.cache.guilds().len()
    ))));
}

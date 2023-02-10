use serenity::{
    client::Context,
    model::{gateway::Activity, guild::Guild, id::UserId},
};

use log::error;

pub async fn senddm(ctx: &Context, guild: Guild, is_new: bool) {
    if !is_new {
        return;
    }
    let teddypicker = UserId(653157614452211712);
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

pub async fn set_bot_status(ctx: &Context) {
    ctx.set_activity(Activity::playing(format!(
        "이모지 확대용 봇 | {}개의 서버에서 일하는중",
        ctx.cache.guilds().len()
    )))
    .await;
}

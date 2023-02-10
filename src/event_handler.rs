use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        gateway::{Activity, Ready},
        guild::Guild,
    },
};

pub struct DiscordEventHandler;

use super::events::{guildbotadd, scene_core};

#[async_trait]

impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{}으로 로그인 완료!", ready.user.tag());

        ctx.set_activity(Activity::playing(format!(
            "이모지 확대용 봇 | {}개의 서버에서 일하는중",
            ctx.cache.guilds().len()
        )))
        .await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        tokio::join!(
            guildbotadd::senddm(&ctx, guild, is_new),
            guildbotadd::set_bot_status(&ctx)
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        tokio::join!(scene_core::send_transfered_image(&ctx, &msg));
    }
}

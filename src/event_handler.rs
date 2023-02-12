use serenity::{
    async_trait,
    client::{Context, EventHandler},
    gateway::ActivityData,
    model::{application::Interaction, channel::Message, gateway::Ready, guild::Guild},
};

use crate::command_handler::handler::seperate_command;
use crate::command_handler::update_command::update_cmds::update_command;

pub struct DiscordEventHandler;

use crate::events::{autosend, guildbotadd};

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{}으로 로그인 완료!", ready.user.tag());

        ctx.set_activity(Some(ActivityData::playing(format!(
            "이모지 확대용 봇 | {}개의 서버에서 일하는중",
            ctx.cache.guilds().len()
        ))));
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        tokio::join!(
            guildbotadd::senddm(&ctx, &guild, is_new),
            guildbotadd::set_bot_status(&ctx),
            guildbotadd::new_guild_added(&ctx, &guild, is_new)
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        tokio::join!(autosend::auto_send_transfered_image(&ctx, &msg));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                /* 봇 업데이트 및 처음 초기 세팅 관련 명령어 */
                "update" => update_command(command, &ctx).await,
                _ => seperate_command(command, &ctx).await,
            }
        }
    }
}

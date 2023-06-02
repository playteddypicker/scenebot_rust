use serenity::{
    all::PremiumTier,
    async_trait,
    builder::{CreateCommand, CreateCommandOption, EditInteractionResponse},
    client::Context,
    model::{
        application::{CommandDataOption, CommandInteraction, CommandOptionType},
        guild::PartialGuild,
        permissions::Permissions,
        prelude::Message,
    },
    Error,
};

use crate::command_handler::explicit_command_list::CommandInterface;
use crate::utils::scene_core::webp_transfer;

use log::info;

struct WebPTransfer;

pub fn get_command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(WebPTransfer)
}

#[async_trait]
impl CommandInterface for WebPTransfer {
    async fn run(
        &self,
        ctx: &Context,
        options: &[CommandDataOption],
        command: &CommandInteraction,
    ) -> Result<Message, Error> {
        let image_id = options[0].value.as_attachment_id().unwrap();
        let image = command.data.resolved.attachments.get(&image_id).unwrap();

        //webp 파일이 아니면 reject함
        let edit_response = if !image.url.ends_with(".webp") {
            EditInteractionResponse::new().content("올바른 webp 이미지를 첨부해주세요.")
        } else {
            let max_size_mb = match PartialGuild::get(&ctx.http, command.guild_id.unwrap())
                .await
                .unwrap()
                .premium_tier
            {
                PremiumTier::Tier3 => 50,
                PremiumTier::Tier2 => 25,
                _ => 10,
            };

            if image.size > 1024 * 1024 * 10 {
                EditInteractionResponse::new().content("10MB 미만의 WebP 이미지만 변환 가능합니다.")
            } else {
                match webp_transfer(image.url.clone(), false).await {
                    Ok(transfered) => {
                        info!(
                            "trasnfered size : {:?}",
                            transfered.data.len() as f64 / 1024.0 / 1024.0
                        );
                        if transfered.data.len() > 1024 * 1024 * max_size_mb {
                            drop(transfered);
                            EditInteractionResponse::new()
                            .content("변환된 이미지의 크기가 너무 큽니다.\n용량 제한을 늘리려면 서버 부스트를 이용해주세요.\n최대 용량 제한은 레벨 3 부스트 기준 100MB입니다.")
                        } else {
                            EditInteractionResponse::new().new_attachment(transfered)
                        }
                    }
                    Err(webperror) => {
                        EditInteractionResponse::new().content(webperror.get_error_message())
                    }
                }
            }
        };

        command.edit_response(&ctx.http, edit_response).await
    }

    fn name(&self) -> String {
        String::from("webp")
    }

    fn register(&self) -> CreateCommand {
        let options = Vec::from([CreateCommandOption::new(
            CommandOptionType::Attachment,
            "webp_image",
            "변환할 WebP 이미지를 첨부해주세요",
        )
        .required(true)]);
        CreateCommand::new(self.name())
            .description("WebP 이미지를 gif나 png로 변환해 전송합니다")
            .set_options(options)
            .default_member_permissions(Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS)
    }
}

use serenity::{
    async_trait,
    builder::{CreateCommand, CreateCommandOption, EditInteractionResponse},
    client::Context,
    model::{
        application::{CommandDataOption, CommandInteraction, CommandOptionType},
        permissions::Permissions,
        prelude::Message,
    },
    Error,
};

use crate::command_handler::explicit_command_list::CommandInterface;
use crate::utils::scene_core::webp_transfer;

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

        let transfered = webp_transfer(image.url.clone()).await;

        command
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().new_attachment(transfered),
            )
            .await
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

use serenity::{
    async_trait,
    builder::{CreateCommand, CreateEmbed},
    client::Context,
    model::{
        application::{CommandDataOption, CommandInteraction},
        prelude::Message,
    },
    Error,
};

use crate::command_handler::explicit_command_list::CommandInterface;
use crate::utils::frameworks::reaction_pages;

struct Help;

pub fn get_command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Help)
}

#[async_trait]
impl CommandInterface for Help {
    async fn run(
        &self,
        ctx: &Context,
        _options: &[CommandDataOption],
        command: &CommandInteraction,
    ) -> Result<Message, Error> {
        reaction_pages::reaction_pages(command.clone(), ctx, get_help_embed()).await
    }

    fn name(&self) -> String {
        String::from("help")
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("이 봇의 사용법을 알려드립니다")
            .clone()
    }
}

fn get_help_embed() -> Vec<CreateEmbed> {
    vec![
        //1페이지 : /update 커맨드 설명
        CreateEmbed::default()
            .title("봇 사용법 1 : 명령어 등록")
            .field(
                "/update 명령어로 명령어 등록하기",
                "봇의 모든 기능을 사용하기 위해서는 서버로부터 명령어를 먼저 가져와야 합니다.".to_owned()+ "\n" +
                "/update 명령어를 누른 후 등록 버튼을 누르면 명령어를 등록할 수 있습니다.",
                false
            )
            .image("https://media.discordapp.net/attachments/1258021816283304027/1258022335320162394/Screenshot_2024-07-03_at_20.png"),
        //2페이지 : /config 명령어
        CreateEmbed::default()
            .title("봇 사용법 2 : /config 명령어")
            .field(
                "/config 명령어로 봇 설정하기",
                "/config 명령어로 이모지 봇 설정을 할 수 있습니다.".to_owned() + "\n" +
                "- \"자동 이모지 크기 조절 켜거나 끄기\" : 켜져있으면 사용자가 이모지를 보낼 때마다 설정된 크기로 이모지를 확대합니다." + "\n" +
                "- \"크기 기본값 설정하기\" : 자동 이모지 크기 조절이 켜져있을 때의 확대값을 설정합니다.",
                false
            )
            .image("https://media.discordapp.net/attachments/1258021816283304027/1258022587473199249/Screenshot_2024-07-03_at_20.32.29.png"),
        //3페이지 : /send 명령어
        CreateEmbed::default()
            .title("봇 사용법 3 : /send 명령어")
            .field(
                "/send 명령어로 원하는 크기로 이모지 전송하기",
                "/send 명령어로 원하는 크기로 이모지를 확대 혹은 축소해 전송할 수 있습니다.".to_owned() + "\n" +
                "니트로가 없는 사용자도 입력값으로 이모지 이름(이미지 예시로는 :kalbrr:)을 입력하면 움짤 이모지를 전송할 수 있습니다." + "\n" +
                "움짤 이모지 크기 조절 기능은 현재는 구현되어있지 않지만, 추후 업데이트 예정입니다.",
                false
            )
            .image("https://media.discordapp.net/attachments/1258021816283304027/1258023032681922591/Screenshot_2024-07-03_at_20.34.16.png")
    ]
}

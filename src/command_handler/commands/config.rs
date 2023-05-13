use serenity::{
    async_trait,
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, EditInteractionResponse, CreateAttachment, CreateInteractionResponseFollowup,
    },
    client::Context,
    futures::StreamExt,
    model::{
        application::{ButtonStyle, CommandDataOption, CommandInteraction},
        permissions::Permissions,
        prelude::Message,
    },
    Error,
};

use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client as DBClient,
    bson::doc,
    Collection
};

use bson::Document;

use crate::command_handler::explicit_command_list::CommandInterface;
use crate::utils::scene_core::ImageSize;
use crate::GlobalGuildConfigs;

use log::error;

use std::{time::Duration, env};

const EXAMPLE_IMAGES: [&str; 5] = [
    "https://cdn.discordapp.com/emojis/1075091165285715968.webp?size=16",
    "https://cdn.discordapp.com/emojis/1075091165285715968.webp?size=64",
    "https://cdn.discordapp.com/emojis/1075091165285715968.webp?size=128",
    "https://media.discordapp.net/attachments/892675167208235058/1075091557746757662/resized.png",
    "https://media.discordapp.net/attachments/892675167208235058/1075091654308024330/resized.png",
];

struct GuildConfigSetting;

pub fn get_command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(GuildConfigSetting)
}

#[async_trait]
impl CommandInterface for GuildConfigSetting {
    async fn run(
        &self,
        ctx: &Context,
        _options: &[CommandDataOption],
        command: &CommandInteraction,
    ) -> Result<Message, Error> {
        let counter_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<GlobalGuildConfigs>()
                .expect("poisened")
                .clone()
        };
        let command = command.clone();
        let mut guilds_config = counter_lock.write().await;
        let guild_config = guilds_config.get_mut(&command.guild_id.unwrap().0);

        match guild_config {
            None => {
                command
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .content("설정 정보를 가져오는데 실패했습니다."),
                    )
                    .await
            }
            Some(gc) => {
                let mut gclock = gc.lock().await;
                if let Err(why) = command
                    .edit_response(
                        &ctx.http,
                        EditInteractionResponse::new()
                            .embed(config_embed(
                                gclock.auto_magnitute_enable,
                                gclock.auto_transfer_webp,
                                gclock.auto_magnitute_config.clone(),
                            ))
                            .components(vec![CreateActionRow::Buttons(Vec::from(
                                config_components(),
                            ))]),
                    )
                    .await
                {
                    error!("Failed to response slash command: {:#?}", why);
                };

                match command.get_response(&ctx.http).await {
                    Ok(msg) => {
                        let mut interaction_stream = msg
                            .await_component_interactions(ctx)
                            .timeout(Duration::from_secs(60 * 5))
                            .filter(move |f| {
                                f.message.id == msg.id
                        //is_some_and 업뎃 후 코드를 다음과 같이 변경
                        // f.member.is_some_and(|&m| m.user.id == interaction.user.id)
                        && f.member.as_ref().unwrap().user.id == command.user.id
                            })
                            .stream();

                        if let Some(button_reaction) = interaction_stream.next().await {
                            match button_reaction.data.custom_id.as_str() {
                                "autoemoji_enabled" => {
                                    if let Err(why) = button_reaction
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::UpdateMessage(
                                                CreateInteractionResponseMessage::new()
                                                    .content(
                                                        match gclock.auto_magnitute_enable { 
                                                            false => {
                                                                gclock.auto_magnitute_enable = true;
                                                                "자동 이모지 크기 조절이 켜졌습니다.\n이제 이모지를 전송하면 설정해둔 크기에 맞게 자동으로 봇이 변환해줍니다." 
                                                            },
                                                            true => {
                                                                gclock.auto_magnitute_enable = false;
                                                                "자동 이모지 크기 조절이 꺼졌습니다."
                                                            } 
                                                        }
                                                    ).components(vec![]).embeds(vec![])
                                            ),
                                        )
                                            .await {
                                                error!("sending error: {:?}", why);
                                            }
                                }
                                "autowebp_enabled" => {
                                if let Err(why) = button_reaction
                                    .create_response(
                                        &ctx.http,
                                        CreateInteractionResponse::UpdateMessage(
                                            CreateInteractionResponseMessage::new()
                                                .content(
                                                    match gclock.auto_transfer_webp { 
                                                        false => {
                                                            gclock.auto_transfer_webp = true;
                                                            "자동 WebP 변환이 켜졌습니다.\n이제 WebP 움짤을 전송하면 자동으로 gif로 변환됩니다." 
                                                        },
                                                        true => {
                                                            gclock.auto_transfer_webp = false;
                                                            "자동 WebP 변환이 꺼졌습니다."
                                                        } 
                                                    }
                                                ).components(vec![]).embeds(vec![])
                                        ),
                                    )
                                        .await {
                                            error!("sending error: {:?}", why);
                                        }
                                }
                                _ => {
                                    if let Err(why) = button_reaction
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::UpdateMessage(
                                                CreateInteractionResponseMessage::new()
                                                    .content(size_notice())
                                                    .components(Vec::from(size_components()))
                                                    .embeds(vec![])
                                                    .add_files(
                                                        [
                                                            CreateAttachment::url(&ctx.http, EXAMPLE_IMAGES[0]).await.unwrap(),
                                                            CreateAttachment::url(&ctx.http, EXAMPLE_IMAGES[1]).await.unwrap(),
                                                            CreateAttachment::url(&ctx.http, EXAMPLE_IMAGES[2]).await.unwrap(),
                                                            CreateAttachment::url(&ctx.http, EXAMPLE_IMAGES[3]).await.unwrap(),
                                                            CreateAttachment::url(&ctx.http, EXAMPLE_IMAGES[4]).await.unwrap(),
                                                        ]
                                                    )
                                            ),
                                        )
                                            .await {
                                                error!("sending error: {:?}", why);
                                            }
                                    let msg = button_reaction.get_response(&ctx.http).await.unwrap();
                                    let mut interaction_stream = msg
                                        .await_component_interactions(ctx)
                                        .timeout(Duration::from_secs(60 * 5))
                                        .filter(move |f| {
                                            f.message.id == msg.id
                                         && f.member.as_ref().unwrap().user.id == command.user.id
                                        }).stream();
                                        
                                    if let Some(sizebutton_reaction) = interaction_stream.next().await {
                                        let size = match sizebutton_reaction.data.custom_id.as_str() {
                                            "setemoji_smallest" => ImageSize::HyperTechniqueOfLisaSuFinger,
                                            "setemoji_small" => ImageSize::Small,
                                            "setemoji_medium" => ImageSize::Medium,
                                            "setemoji_large" => ImageSize::Large,
                                            "setemoji_largest" => ImageSize::HyperSuperUltraSexFeaturedFuckingLarge,
                                            _ => ImageSize::Auto,
                                        };
                                        gclock.auto_magnitute_config = size;
                                        if let Err(why) = sizebutton_reaction
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::UpdateMessage(
                                                CreateInteractionResponseMessage::new()
                                                    .content(
                                                        format!("자동 이모지 변환 사이즈를 {}(으)로 설정했습니다.", 
                                                            match sizebutton_reaction.data.custom_id.as_str() {
                                                                "setemoji_smallest" => "절라 짝게",
                                                                "setemoji_small" => "작게",
                                                                "setemoji_medium" => "중간",
                                                                "setemoji_large" => "크게",
                                                                "setemoji_largest" => "존,나 크게",
                                                                _ => "자동"
                                                            })
                                                    ).components(vec![]).embeds(vec![]).files(vec![])
                                            ),
                                        )
                                        .await {
                                            error!("sending error: {:?}", why);
                                        }

                                    }
                                }
                            }
                        }
                        //ㅅㅂ 지금은 어쩔수없음. 일단 커맨드에 fetch하는식은 나중으로 미루고
                        //일단은 이렇게 조치함.
                        let client_uri = env::var("DB_URI").expect("couldn't' find db uri");
                        let options =
                            ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
                                .await.unwrap();

                        let db_client = DBClient::with_options(options).unwrap();

                        let collections: Collection<Document> = db_client.database("scene").collection(env::var("BOT_DB_NAME").unwrap().as_str());
                        collections.find_one_and_update(
                            doc! {
                                "guild_id" : gclock.guild_id.get() as f64
                            }, 
                            doc! {
                                "$set" : {
                                    "guild_id" : gclock.guild_id.get() as f64,
                                    "auto_magnitute_enable" : gclock.auto_magnitute_enable,
                                    "auto_magnitute_config" : ImageSize::value_to_string(&(gclock.auto_magnitute_config)),
                                    "auto_transfer_webp" : gclock.auto_transfer_webp
                                }
                            }, None
                        ).await.unwrap();
                        
                
                        Ok(msg)
                    }
                    Err(why) => {
                        error!("Couldn't get message info from interaction.\n{:#?}", why);
                        Err(why)
                    }
                }
            }
        }
    }

    fn name(&self) -> String {
        String::from("config")
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("이 서버의 봇 설정을 편집해요")
            .default_member_permissions(Permissions::ADMINISTRATOR)
    }
}

fn config_embed(autoemoji: bool, autowebpsend: bool, default_emoji_size: ImageSize) -> CreateEmbed {
    CreateEmbed::new()
        .title("봇 설정")
        .description("설정하고 싶은 것을 선택해주세요")
        .fields([
            (
                "자동 이모지 크기 조절 설정 : 켜져있으면 끄고, 꺼져있으면 킵니다.",
                match autoemoji {
                    true => "현재 상태 : 켜짐",
                    false => "현재 상태 : 꺼짐",
                },
                false,
            ),
            (
                "자동 WebP 변환 전송 설정 : 켜져있으면 끄고, 꺼져있으면 킵니다.",
                match autowebpsend {
                    true => "현재 상태 : 켜짐",
                    false => "현재 상태 : 꺼짐",
                },
                false,
            ),
            (
                "이모지 크기 조정 기본값",
                match default_emoji_size {
                    ImageSize::HyperTechniqueOfLisaSuFinger => "절라 짝음",
                    ImageSize::Small => "작음",
                    ImageSize::Medium => "적당함",
                    ImageSize::Large => "큼",
                    ImageSize::HyperSuperUltraSexFeaturedFuckingLarge => "절라 큼",
                    ImageSize::Auto => "자동",
                },
                false,
            ),
        ])
        .color((255, 255, 255))
}

fn config_components() -> [CreateButton; 3] {
    [
        CreateButton::new("autoemoji_enabled")
            .label("자동 이모지 크기 조절 켜거나 끄기")
            .style(ButtonStyle::Primary),
            CreateButton::new("autowebp_enabled")
            .label("WebP 자동 변환 전송 켜거나 끄기")
            .style(ButtonStyle::Primary),
        CreateButton::new("set_default_autoemoji_size")
            .label("크기 기본값 설정하기")
            .style(ButtonStyle::Secondary),
    ]
}

fn size_notice() -> String {
    "
**__설정하고싶은 크기를 선택해주세요.__**\n\n
순서대로 \"**절라 짝게**\", \"**작게**\", \"**중간**\",\"**큼**\", \"**절라 큼**\" 입니다.\n
이 과정은 이미지 크기 조절 알고리즘을 거치기 때문에 설정 크기가 클수록 응답 속도도 늦어집니다.\n\n
\"자동\"은 이모지의 원래 크기에 따라 자동으로 바꿔집니다.\n
즉, 이미지 크기 변환 과정이 없어 전송 속도가 가장 빠릅니다.\n
    ".to_string()
}

fn size_components() -> [CreateActionRow; 2] {
    [CreateActionRow::Buttons(vec![
        CreateButton::new("setemoji_smallest")
            .label("절라 짝게")
            .style(ButtonStyle::Danger),
        CreateButton::new("setemoji_small")
            .label("작게")
            .style(ButtonStyle::Secondary),
        CreateButton::new("setemoji_medium")
            .label("중간")
            .style(ButtonStyle::Secondary),
        CreateButton::new("setemoji_large")
            .label("크게")
            .style(ButtonStyle::Secondary),
        CreateButton::new("setemoji_largest")
            .label("존,나크게")
            .style(ButtonStyle::Danger),
    ]), CreateActionRow::Buttons(vec![
        CreateButton::new("setemoji_auto")
            .label("자동")
            .style(ButtonStyle::Primary),
    ])]
}

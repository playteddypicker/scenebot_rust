use serenity::{
    //async_trait,
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, EditInteractionResponse, EditMessage,
    },
    client::Context,
    futures::StreamExt,
    model::{
        application::{ButtonStyle, CommandInteraction},
        id::GuildId,
    },
};

use log::error;

use std::time::Duration;

use super::super::explicit_command_list::COMMAND_LIST;
use super::update_checker::{check_updates, UpdateStatus};

pub async fn update_command(command: CommandInteraction, ctx: &Context) {
    //메시지 응답 타입은 3개임
    //defer 후 응답하는식으로
    //이건 나중에 error리턴하는 구조체 하나 만들어서 따로 핸들링..
    command.defer(&ctx.http).await.unwrap();

    match command.guild_id {
        Some(gid) => match check_updates(ctx, gid).await {
            //1. 서버에서 처음 쓸때 (커맨드가 update말곤 존재하지 않음
            UpdateStatus::FirstSetting => first_setup_msg(ctx, gid, command).await,
            UpdateStatus::LatestVersion => latest_version_msg(ctx, command).await,
            UpdateStatus::UpdateAvailable(unassigned_commands) => {
                update_available_msg(ctx, gid, unassigned_commands, command).await
            }
            UpdateStatus::FailedtoLoad => failed_notice_msg(ctx, command).await,
        },
        None => failed_notice_msg(ctx, command).await,
    }
}

async fn first_setup_msg(ctx: &Context, gid: GuildId, command: CommandInteraction) {
    //먼저 안내용 임베드하고 버튼먼저 보냄
    //defer되어있으니 edit_original_interaction_response로 해야함
    if let Err(why) = command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embed(setup_embed())
                .components(vec![CreateActionRow::Buttons(Vec::from(
                    update_components(false),
                ))]),
        )
        .await
    {
        error!("Failed to response slash command: {:#?}", why);
    };

    //버튼 누를때까지 기다림
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
                    "update_cmds" => {
                        match button_reaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .embed(
                                            CreateEmbed::new()
                                                .title("서버로부터 명령어를 등록하는 중..."),
                                        )
                                        .components(vec![CreateActionRow::Buttons(Vec::from(
                                            update_components(true),
                                        ))]),
                                ),
                            )
                            .await
                        {
                            Ok(_) => {
                                //register_commands 메소드로 전부 등록
                                COMMAND_LIST.register_commands(gid, ctx).await;
                                if let Err(why) = button_reaction
                                    .edit_response(
                                        &ctx,
                                        EditInteractionResponse::new().embed(
                                            CreateEmbed::new()
                                                .title("명령어 등록이 완료되었습니다."),
                                        ),
                                    )
                                    .await
                                {
                                    error!("Couldn't send complete msg. {:#?}", why);
                                }
                            }
                            Err(why) => {
                                error!("Couldn't edit response msg., {:#?}", why);
                            }
                        }
                    }
                    "show_patchnotes" => {
                        //패치노트 임베드 보내기
                    }
                    _ => {
                        command.delete_response(&ctx.http).await.unwrap();
                    }
                }
            }
        }
        Err(why) => {
            error!("Couldn't get message info from interaction.\n{:#?}", why);
        }
    }
}

async fn latest_version_msg(ctx: &Context, command: CommandInteraction) {
    //먼저 안내용 임베드하고 버튼먼저 보냄
    if let Err(why) = command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content("✅ 이 서버에는 업데이트 할 명령어가 없습니다."),
        )
        .await
    {
        error!("Failed to response slash command: {:#?}", why);
    };
}

async fn update_available_msg(
    ctx: &Context,
    gid: GuildId,
    unassigned_commands: Vec<String>,
    command: CommandInteraction,
) {
    if let Err(why) = command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embed(
                    CreateEmbed::new()
                        .title("ℹ️ 아직 등록되지 않은 명령어가 있습니다")
                        .description(unassigned_commands.join("\n")),
                )
                .components(vec![CreateActionRow::Buttons(Vec::from(
                    update_components(false),
                ))]),
        )
        .await
    {
        error!("Failed to response slash command: {:#?}", why);
    };

    //버튼 누를때까지 기다림
    match command.get_response(&ctx.http).await {
        Ok(mut msg) => {
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
                    "update_cmds" => {
                        match button_reaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .embed(
                                            CreateEmbed::new()
                                                .title("서버로부터 명령어를 등록하는 중..."),
                                        )
                                        .components(vec![CreateActionRow::Buttons(Vec::from(
                                            update_components(true),
                                        ))]),
                                ),
                            )
                            .await
                        {
                            Ok(_) => {
                                //register_commands 메소드로 전부 등록
                                COMMAND_LIST.register_commands(gid, ctx).await;

                                if let Err(why) = msg
                                    .edit(
                                        &ctx.http,
                                        EditMessage::new().embed(
                                            CreateEmbed::new()
                                                .title("명령어 등록이 완료되었습니다."),
                                        ),
                                    )
                                    .await
                                {
                                    error!("Couldn't send complete msg. {:#?}", why);
                                }
                            }
                            Err(why) => {
                                error!("Couldn't edit response msg., {:#?}", why);
                            }
                        }
                    }
                    "show_patchnotes" => {
                        button_reaction
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("그런거 없다 게이야 ㅋ"),
                                ),
                            )
                            .await
                            .unwrap();
                    }
                    _ => {
                        command.delete_response(&ctx.http).await.unwrap();
                    }
                }
            }
        }
        Err(why) => {
            error!("Couldn't get message info from interaction.\n{:#?}", why);
        }
    }
}

async fn failed_notice_msg(ctx: &Context, command: CommandInteraction) {
    if let Err(why) = command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content("⚠️ 서버 정보를 불러오는 데 실패했습니다."),
        )
        .await
    {
        error!("Failed to reponse slash command: {:#?}", why);
    }
}

//나중에 캐릭터별로 대사 다르게 해야하니까..
//색상도 다르게 설정
fn setup_embed() -> CreateEmbed {
    CreateEmbed::new()
        .title("이 서버에는 명령어가 아직 등록되어있지 않습니다.")
        .description("밑의 등록 버튼을 눌러 서버에 있는 명령어를 불러와 등록할 수 있어요")
        .color((255, 255, 255))
}

fn update_components(pressed: bool) -> [CreateButton; 3] {
    [
        CreateButton::new("update_cmds")
            .label("업데이트")
            .style(ButtonStyle::Primary)
            .disabled(pressed),
        CreateButton::new("show_patchnotes")
            .label("패치노트")
            .style(ButtonStyle::Secondary)
            .disabled(pressed),
        CreateButton::new("cancel_update")
            .label("안할래")
            .style(ButtonStyle::Danger)
            .disabled(pressed),
    ]
}

use serenity::{
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, EditInteractionResponse,
    },
    client::Context,
    futures::StreamExt,
    model::{
        application::{ButtonStyle, CommandInteraction},
        channel::ReactionType,
    },
};

use std::time::Duration;

use log::error;

#[derive(Debug)]
pub struct SkippableEmbed {
    total: usize,
    pub(crate) embedlist: Vec<CreateEmbed>,
    current_idx: usize,
    button_disable_option: (bool, bool, bool, bool),
}

impl SkippableEmbed {
    //current_idx가 total보다 작을때만 발생함
    fn next(&mut self) {
        self.current_idx = if self.current_idx + 1 < self.total {
            self.current_idx + 1
        } else {
            self.current_idx
        }
    }

    fn prev(&mut self) {
        self.current_idx = if self.current_idx > 0 {
            self.current_idx - 1
        } else {
            self.current_idx
        }
    }

    fn skip_end(&mut self) {
        self.current_idx = self.total - 1;
    }

    fn skip_start(&mut self) {
        self.current_idx = 0;
    }

    fn check_disable_button(&mut self) {
        self.button_disable_option = if self.current_idx + 1 == self.total {
            // <<, <만 활성화
            (false, false, true, true)
        } else if self.current_idx == 0 {
            //>, >>만 활성화
            (true, true, false, false)
        } else if self.total == 0 {
            //넘길 페이지가 없으므로 전부 비활
            (true, true, true, true)
        } else {
            (false, false, false, false)
        }
    }
}

fn set_reaction_page_action_row(reactive_interaction: &mut SkippableEmbed) -> CreateActionRow {
    //이 함수 호출하기 직전에 check_disable_button을 호출하므로 굳이 밖에서 그럴필요없이
    //그냥 내부에서 호출하는게 나음ㅋ
    reactive_interaction.check_disable_button();

    CreateActionRow::Buttons(vec![
        CreateButton::new("to_start")
            .style(ButtonStyle::Secondary)
            .emoji("⏮️".parse::<ReactionType>().unwrap())
            .disabled(reactive_interaction.button_disable_option.0),
        CreateButton::new("previous")
            .style(ButtonStyle::Secondary)
            .emoji("⬅️".parse::<ReactionType>().unwrap())
            .disabled(reactive_interaction.button_disable_option.1),
        CreateButton::new("next")
            .style(ButtonStyle::Secondary)
            .emoji("➡️".parse::<ReactionType>().unwrap())
            .disabled(reactive_interaction.button_disable_option.2),
        CreateButton::new("to_end")
            .style(ButtonStyle::Secondary)
            .emoji("⏭️".parse::<ReactionType>().unwrap())
            .disabled(reactive_interaction.button_disable_option.3),
        CreateButton::new("remove")
            .style(ButtonStyle::Danger)
            .emoji("✖️".parse::<ReactionType>().unwrap()),
    ])
}

pub async fn reaction_pages(
    interaction: CommandInteraction,
    ctx: &Context,
    embeds: Vec<CreateEmbed>,
) -> Result<serenity::model::channel::Message, serenity::Error> {
    let mut reactive_interaction = SkippableEmbed {
        total: embeds.len(),
        embedlist: embeds,
        current_idx: 0,
        button_disable_option: (true, true, true, true),
    };

    //interaction을 edit해서 먼저 button component를 붙이기
    //나중에 multi-embed framework랑 안겹치게 custom id 설정함
    //
    //전송된 Embed에 component 붙이기
    if let Err(why) = interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().components(vec![set_reaction_page_action_row(
                &mut reactive_interaction,
            )]),
        )
        .await
    {
        error!("an error occured while adding buttons.");
        return Err(why);
    };

    //button interaction 계속 받기. 5분동안만 시간 지나면 Ok() 반환
    match interaction.get_response(&ctx.http).await {
        Ok(msg) => {
            //filter 부분
            let mut interaction_stream = msg
                .await_component_interactions(ctx)
                .timeout(Duration::from_secs(60 * 3))
                .filter(move |f| {
                    f.message.id == msg.id
                        //is_some_and 업뎃 후 코드를 다음과 같이 변경
                        // f.member.is_some_and(|&m| m.user.id == interaction.user.id)
                        && f.member.as_ref().unwrap().user.id == interaction.user.id
                })
                .stream();

            while let Some(button_reaction) = interaction_stream.next().await {
                match button_reaction.data.custom_id.as_str() {
                    "to_start" => reactive_interaction.skip_start(),
                    "next" => reactive_interaction.next(),
                    "previous" => reactive_interaction.prev(),
                    "to_end" => reactive_interaction.skip_end(),
                    "remove" => {
                        if let Err(why) = msg.delete(&ctx.http).await {
                            error!("Couldn't delete message in reaction button invoking 'remove'.");
                            return Err(why);
                        }
                        return Ok(msg);
                    }
                    _ => continue,
                }

                if let Err(why) = button_reaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    reactive_interaction.embedlist
                                        [reactive_interaction.current_idx]
                                        .clone(),
                                )
                                .components(vec![set_reaction_page_action_row(
                                    &mut reactive_interaction,
                                )]),
                        ),
                    )
                    .await
                {
                    error!("Couldn't set embed.");
                    return Err(why);
                }
            }

            //dangling interaction 방지로 끝나면 바로 삭제
            if let Err(why) = msg.delete(&ctx.http).await {
                error!("an error occured while deleting message.");
                return Err(why);
            };

            Ok(msg)
        }
        Err(why) => {
            error!("Couldn't get message info from interaction.");
            Err(why)
        }
    }
}

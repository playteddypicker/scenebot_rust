use log::error;
use regex::Regex;
use serenity::{
    builder::{CreateAttachment, CreateMessage},
    client::Context,
    model::channel::{Message, MessageReference},
};

use crate::utils::scene_core::ImageSize::{Auto, HyperTechniqueOfLisaSuFinger, Medium, Small};
use crate::GlobalGuildConfigs;

use crate::utils::scene_core::{get_resized_image, webp_transfer, EmojiFilter};
use std::time::Instant;

impl EmojiFilter for Message {
    fn emoji_format_filter(&self) -> Result<(bool, String), ()> {
        let msg_content_vec: Vec<&str> = self.content.split(':').collect();
        let content_regex: Regex = Regex::new(r"^<a?:.+?:\d+>$").unwrap();
        match self.referenced_message.is_some()
            || self.author.bot
            || !self.attachments.is_empty()
            || !content_regex.is_match(&self.content)
            || !self.mentions.is_empty()
            || msg_content_vec.len() != 3
        {
            false => {
                let mut id = msg_content_vec[2].to_string();
                id.pop();
                let mut is_png = false;
                let img_url = format!(
                    "https://cdn.discordapp.com/emojis/{}.{}",
                    id,
                    if self.content.contains("<a:") {
                        "gif"
                    } else {
                        is_png = true;
                        "webp"
                    }
                );
                Ok((is_png, img_url))
            }
            true => Err(()),
        }
    }
}

pub async fn auto_send_transfered_image(ctx: &Context, msg: &Message) {
    let filtered = msg.emoji_format_filter();
    if filtered.is_err() {
        return;
    }

    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GlobalGuildConfigs>()
            .expect("poisened")
            .clone()
    };

    let guilds_config = counter_lock.read().await;
    let gconfig_lock = guilds_config.get(&msg.guild_id.unwrap().0).unwrap();
    let size_config = {
        let gconfig = gconfig_lock.lock().await;

        if !gconfig.auto_magnitute_enable {
            return;
        }
        gconfig.auto_magnitute_config.clone()
    };

    let delresult = msg.delete(&ctx.http).await;

    if let Err(why) = delresult {
        error!("couldn't delete message. {:?}", why);
    }

    let (is_png, img_url) = filtered.unwrap();

    //webp png로 보낼수있는거
    if let Err(why) = if is_png
        && matches!(
            size_config,
            HyperTechniqueOfLisaSuFinger | Small | Medium | Auto
        ) {
        let size = match size_config {
            HyperTechniqueOfLisaSuFinger => "?size=16",
            Small => "?size=64",
            Medium => "?size=256",
            _ => "",
        };
        msg.channel_id
            .say(
                &ctx.http,
                "**".to_owned()
                    + match &msg.member {
                        Some(m) => match &m.nick {
                            Some(nick) => nick,
                            None => &msg.author.name,
                        },
                        None => &msg.author.name,
                    }
                    + "** :",
            )
            .await
            .unwrap();
        msg.channel_id.say(&ctx.http, img_url + size).await
    } else {
        let files = [if is_png {
            get_resized_image(ctx, img_url.as_str(), &size_config).await
        } else {
            CreateAttachment::url(&ctx.http, img_url.as_str())
                .await
                .unwrap()
        }];
        msg.channel_id
            .send_files(
                &ctx.http,
                files,
                CreateMessage::new().content(
                    "**".to_owned()
                        + match &msg.member {
                            Some(m) => match &m.nick {
                                Some(nick) => nick,
                                None => &msg.author.name,
                            },
                            None => &msg.author.name,
                        }
                        + "** :",
                ),
            )
            .await
    } {
        error!("send message error: {}", why);
    }
}

pub async fn auto_send_webp_image(ctx: &Context, msg: &Message) {
    //webp 아니거나 attachment 없으면 리턴
    if msg.attachments.len() != 1
        || msg.attachments.get(0).is_none()
        || !msg.attachments.get(0).unwrap().url.ends_with(".webp")
    {
        return;
    }

    let counter_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GlobalGuildConfigs>()
            .expect("poisened")
            .clone()
    };

    let guilds_config = counter_lock.write().await;

    let gconfig_lock = guilds_config.get(&msg.guild_id.unwrap().0).unwrap();
    let gconfig = gconfig_lock.lock().await;
    if !gconfig.auto_transfer_webp {
        return;
    }

    if msg.attachments[0].size > 1024 * 1024 * 10 {
        return;
    }

    let now = Instant::now();

    if let Ok(transfered) = webp_transfer(msg.attachments[0].url.clone(), true).await {
        if let Err(why) = msg
            .channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new()
                    .content(format!(
                        "변환 시간 : {:.4}초",
                        now.elapsed().as_millis() as f64 / 1000.0
                    ))
                    .add_file(transfered)
                    .reference_message(MessageReference::from(msg)),
            )
            .await
        {
            error!("auto webp transfer error. {:?}", why);
        }
    }
}

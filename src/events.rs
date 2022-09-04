use regex::Regex;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        gateway::{Activity, Ready},
        guild::Guild,
        id::UserId,
    },
};

pub struct DiscordEventHandler;

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
            Err(why) => println!("error occured while creating dm channel: {:?}", why),
        }

        ctx.set_activity(Activity::playing(format!(
            "이모지 확대용 봇 | {}개의 서버에서 일하는중",
            ctx.cache.guilds().len()
        )))
        .await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let msg_content_vec: Vec<&str> = msg.content.split(':').collect();
        let content_regex: Regex = Regex::new(r"^<a?:.+?:\d+>$").unwrap();

        if msg.referenced_message.is_some()
            || msg.author.bot
            || !msg.attachments.is_empty()
            || !content_regex.is_match(&msg.content)
            || !msg.mentions.is_empty()
            || msg_content_vec.len() != 3
        {
            return;
        }

        let mut id = msg_content_vec[2].to_string();
        id.pop();

        msg.delete(&ctx.http)
            .await
            .expect("couldn't delete message.");

        let files = [&format!(
            "https://cdn.discordapp.com/emojis/{}.{}",
            id,
            if msg.content.contains("<a:") {
                "gif"
            } else {
                "png"
            }
        )[..]];

        if let Err(why) = msg
            .channel_id
            .send_files(&ctx.http, files, |m| {
                m.content(
                    "**".to_owned()
                        + &match msg.member {
                            Some(m) => match m.nick {
                                Some(nick) => nick,
                                None => msg.author.name,
                            },
                            None => msg.author.name,
                        }
                        + "** :",
                )
            })
            .await
        {
            println!("send message error: {}", why);
        };
    }
}

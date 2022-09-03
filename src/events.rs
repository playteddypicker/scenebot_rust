use regex::Regex;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, gateway::Ready, guild::Guild, id::GuildId},
};

pub struct DiscordEventHandler;

#[async_trait]

impl EventHandler for DiscordEventHandler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{}으로 로그인 완료!", ready.user.tag());
    }

    //async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {}

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

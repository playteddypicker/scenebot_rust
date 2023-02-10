use log::error;
use regex::Regex;
use serenity::{client::Context, model::channel::Message};

use std::num::NonZeroU32;

use fast_image_resize as fr;

pub async fn send_transfered_image(ctx: &Context, msg: &Message) {
    let filtered = emoji_format_filter(msg);
    if filtered.is_err() {
        return;
    }

    let mut id = filtered.unwrap();
    id.pop();

    if let Err(why) = msg.delete(&ctx.http).await {
        error!("couldn't delete message. {:?}", why);
    }

    let img_url = format!(
        "https://cdn.discordapp.com/emojis/{}.{}",
        id,
        if msg.content.contains("<a:") {
            "gif"
        } else {
            "png"
        }
    );

    let (reqtime, transtime, savetime) = resize_png(&img_url).await;

    let files = [&img_url, "temp.png"];

    if let Err(why) = msg
        .channel_id
        .send_files(&ctx.http, files, |m| {
            m.content(
                format!("걸린 시간\n이미지 주소에서 받는 시간 : {reqtime}초\n이미지 변환 시간 : {transtime}초\n이미지 저장 시간 : {savetime}초\n**")
                    + match &msg.member {
                        Some(m) => match &m.nick {
                            Some(nick) => nick,
                            None => &msg.author.name,
                        },
                        None => &msg.author.name,
                    }
                    + "** :",
            )
        })
        .await
    {
        error!("send message error: {}", why);
    };
}

fn emoji_format_filter(msg: &Message) -> Result<String, ()> {
    let msg_content_vec: Vec<&str> = msg.content.split(':').collect();
    let content_regex: Regex = Regex::new(r"^<a?:.+?:\d+>$").unwrap();

    match msg.referenced_message.is_some()
        || msg.author.bot
        || !msg.attachments.is_empty()
        || !content_regex.is_match(&msg.content)
        || !msg.mentions.is_empty()
        || msg_content_vec.len() != 3
    {
        false => Ok(msg_content_vec[2].to_string()),
        true => Err(()),
    }
}

enum ImageSize {
    Small,
    Medium,
    Large,
    SuperLarge,
    HyperSuperUltraSexFeaturedFuckingLarge,
}

async fn resize_png(img_url: &str) -> (f64, f64, f64) {
    let now = std::time::SystemTime::now();
    let img = image::load_from_memory(&reqwest::get(img_url).await.unwrap().bytes().await.unwrap())
        .unwrap();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let a = now.elapsed().unwrap().as_millis() as f64 / 1000.0;

    let src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(1024).unwrap();
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    let mut dst_view = dst_image.view_mut();

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();
    let b = now.elapsed().unwrap().as_millis() as f64 / 1000.0;

    //tokio::fs::File::create("temp.png").await.unwrap();

    image::save_buffer(
        "temp.png",
        dst_image.buffer(),
        1024,
        1024,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let c = now.elapsed().unwrap().as_millis() as f64 / 1000.0;

    (a, b, c)
}

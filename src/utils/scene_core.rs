use serde::{Deserialize, Serialize};
use serenity::{builder::CreateAttachment, client::Context};

use std::io::{BufReader, BufWriter, Cursor, Seek, Write};
use std::num::NonZeroU32;

use fast_image_resize as fr;
use image::codecs::{gif::GifEncoder, webp::WebPDecoder};
use image::{io::Reader, AnimationDecoder, ImageEncoder};

//png인지 확인하는 부울값과 img url을 반환함
pub trait EmojiFilter {
    fn emoji_format_filter(&self) -> Result<(bool, String), ()>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ImageSize {
    HyperTechniqueOfLisaSuFinger,           //16x16
    Small,                                  //64x64
    Medium,                                 //기본 사이즈(128x128)
    Large,                                  //256x256
    HyperSuperUltraSexFeaturedFuckingLarge, //300x300
    Auto,
}

impl ImageSize {
    pub fn string_to_value(input_str: &str) -> Self {
        match input_str {
            "HyperTechniqueOfLisaSuFinger" => Self::HyperTechniqueOfLisaSuFinger,
            "Small" => Self::Small,
            "Medium" => Self::Medium,
            "Large" => Self::Large,
            "HyperSuperUltraSexFeaturedFuckingLarge" => {
                Self::HyperSuperUltraSexFeaturedFuckingLarge
            }
            _ => Self::Auto,
        }
    }

    pub fn value_to_string(input_value: &ImageSize) -> String {
        let st = match input_value {
            Self::HyperTechniqueOfLisaSuFinger => "HyperTechniqueOfLisaSuFinger",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::HyperSuperUltraSexFeaturedFuckingLarge => {
                "HyperSuperUltraSexFeaturedFuckingLarge"
            }
            Self::Auto => "Auto",
        };
        st.to_string()
    }
}

//img_url은 항상 PNG파일임
pub async fn get_resized_image(
    ctx: &Context,
    img_url: &str,
    img_size: &ImageSize,
) -> CreateAttachment {
    match img_size {
        ImageSize::Auto => CreateAttachment::url(ctx.http.clone(), img_url)
            .await
            .unwrap(),
        ImageSize::HyperTechniqueOfLisaSuFinger => {
            resize_png(
                img_url,
                NonZeroU32::new(16).unwrap(),
                NonZeroU32::new(16).unwrap(),
            )
            .await
        }
        ImageSize::Small => {
            resize_png(
                img_url,
                NonZeroU32::new(64).unwrap(),
                NonZeroU32::new(64).unwrap(),
            )
            .await
        }
        ImageSize::Medium => {
            resize_png(
                img_url,
                NonZeroU32::new(128).unwrap(),
                NonZeroU32::new(128).unwrap(),
            )
            .await
        }
        ImageSize::Large => {
            resize_png(
                img_url,
                NonZeroU32::new(256).unwrap(),
                NonZeroU32::new(256).unwrap(),
            )
            .await
        }
        ImageSize::HyperSuperUltraSexFeaturedFuckingLarge => {
            resize_png(
                img_url,
                NonZeroU32::new(300).unwrap(),
                NonZeroU32::new(300).unwrap(),
            )
            .await
        }
    }
}

#[tokio::test]
async fn test() {
    webp_transfer().await;
}

pub async fn webp_transfer() {
    let mut reader_buf = Cursor::new(Vec::new());

    reader_buf.write_all(&reqwest::get(
            "https://cdn.discordapp.com/attachments/753490327255515176/1090414871243399289/1668529071.webp"
            )
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap().to_vec()[..]
        ).unwrap();

    println!("asdf");

    reader_buf.rewind().unwrap();
    println!("qwer");

    let decoded_webp = WebPDecoder::new(reader_buf).unwrap();
    //let mut result_buf = BufWriter::new(Vec::new());
    let mut result_buf = std::fs::File::create("out.gif").unwrap();

    //default extension
    let extension = if decoded_webp.has_animation() {
        let frames = decoded_webp.into_frames();
        GifEncoder::new(&mut result_buf)
            .try_encode_frames(frames)
            .unwrap();
        println!("transfered webp to gif.");
        "gif"
    } else {
        "png"
    };

    /*
    CreateAttachment::bytes(
        result_buf.into_inner().unwrap().to_vec(),
        "transfered".to_string() + extension,
    )*/
}

use image::{codecs::png::PngEncoder, ColorType};

async fn resize_png(
    img_url: &str,
    dst_width: NonZeroU32,
    dst_height: NonZeroU32,
) -> CreateAttachment {
    let img = image::load_from_memory(&reqwest::get(img_url).await.unwrap().bytes().await.unwrap())
        .unwrap();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();

    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    let mut dst_view = dst_image.view_mut();

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear));
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )
        .unwrap();

    CreateAttachment::bytes(
        result_buf.into_inner().unwrap().to_vec(),
        "resized.png".to_string(),
    )
}

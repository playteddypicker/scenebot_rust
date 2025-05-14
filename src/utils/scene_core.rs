use serde::{Deserialize, Serialize};
use serenity::{builder::CreateAttachment, client::Context};

use std::io::BufWriter;
use std::num::NonZeroU32;

use fast_image_resize as fr;

use image::ImageEncoder;

use regex::Regex;

//png인지 확인하는 부울값과 img url을 반환함
pub trait EmojiFilter {
    fn emoji_format_filter(&self) -> Result<(bool, String), ()>;
    fn double_emoji_format_filter(&self) -> Result<(bool, String, String), ()>;
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
        //여기부터는 작동 안함. dead code인데.. 나중에 고치기
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
        //여기까지 dead code
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

pub enum WebPTransferError {
    GetRequestFailed,
    DecodingWebPError,
    GifEncodingError,
    SetRepeatError,
    SizeLimitExceeded,
    AutoPngNotNeeded,
    Mollu,
}

impl WebPTransferError {
    pub fn get_error_message(&self) -> String {
        match self {
            Self::GetRequestFailed => "디스코드 서버로부터 이미지를 가져오는 데 실패했습니다.",
            Self::DecodingWebPError => "WebP 이미지를 디코딩하는데 실패했습니다.",
            Self::GifEncodingError => "WebP 이미지를 Gif 이미지로 인코딩하는데 실패했습니다.",
            Self::SetRepeatError => "Gif 반복 설정을 하는데 실패했습니다.",
            Self::SizeLimitExceeded => {
                "변환하려는 WebP의 크기가 너무 큽니다. 2MB 이하의 WebP 이미지만 지원합니다."
            }
            Self::AutoPngNotNeeded => "정적 webp는 지원하니까 굳이..?", //리팩토링할때 디코더 -
            //필터 - 인코더 순으로 다시
            Self::Mollu => "에러났는데 뭔지모르겠노",
        }
        .to_string()
    }
}

pub fn emoji_format_filter(emoji_string: &str) -> Result<(bool, String), ()> {
    let msg_content_vec: Vec<&str> = emoji_string.split(':').collect();
    let content_regex: Regex = Regex::new(r"^<a?:.+?:\d+>$").unwrap();
    match !content_regex.is_match(emoji_string) || msg_content_vec.len() != 3 {
        false => {
            let mut id = msg_content_vec[2].to_string();
            id.pop();
            let mut is_png = false;
            let img_url = format!(
                "https://cdn.discordapp.com/emojis/{}.{}",
                id,
                if emoji_string.contains("<a:") {
                    "gif"
                } else {
                    is_png = true;
                    "png"
                }
            );
            Ok((is_png, img_url))
        }
        true => Err(()),
    }
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

pub async fn merge_two_emojis(
    first_url: &str,
    second_url: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use image::{imageops, GenericImageView, ImageFormat};
    use std::io::Cursor;

    // 첫 번째 이모지 URL 및 두 번째 이모지 URL 생성 (크기 128x128)
    let first_emoji_url = format!("{}?size=128", first_url);
    let second_emoji_url = format!("{}?size=128", second_url);

    // 두 이모지를 병렬로 가져오기
    let (first_emoji_result, second_emoji_result) = tokio::join!(
        async {
            reqwest::get(&first_emoji_url)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                .bytes()
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        },
        async {
            reqwest::get(&second_emoji_url)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                .bytes()
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }
    );

    // 결과 처리
    let first_emoji_bytes = first_emoji_result?.to_vec();
    let second_emoji_bytes = second_emoji_result?.to_vec();

    // 이미지 로드
    let first_img = image::load_from_memory(&first_emoji_bytes)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let second_img = image::load_from_memory(&second_emoji_bytes)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    // 첫 번째와 두 번째 이모지의 크기 확인
    let (width1, height1) = first_img.dimensions();
    let (width2, height2) = second_img.dimensions();

    // 새 이미지 생성 (두 이모지를 나란히 배치)
    let mut merged_img = image::RgbaImage::new(width1 + width2, std::cmp::max(height1, height2));

    // 첫 번째 이모지 복사
    imageops::overlay(&mut merged_img, &first_img, 0, 0);

    // 두 번째 이모지 복사 (첫 번째 이모지 오른쪽에 배치)
    imageops::overlay(&mut merged_img, &second_img, width1 as i64, 0);

    // 결과 이미지를 PNG로 인코딩
    let mut result_buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut result_buffer);
        merged_img
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    }

    Ok(result_buffer)
}

use serde::{Deserialize, Serialize};
use serenity::{builder::CreateAttachment, client::Context};

use std::io::{BufReader, BufWriter, Cursor, Seek, Write};
use std::num::NonZeroU32;

use fast_image_resize as fr;
use image::codecs::{
    gif::{GifEncoder, Repeat},
    webp::WebPDecoder,
};
use image::{AnimationDecoder, ImageDecoder, ImageEncoder};

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

pub async fn webp_transfer(image_url: String) -> CreateAttachment {
    use std::time::Instant;
    let now = Instant::now();

    println!("started getting webp image data from url.");
    let mut reader_buf = Cursor::new(Vec::new());

    reader_buf
        .write_all(
            &reqwest::get(image_url)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec()[..],
        )
        .unwrap();

    println!(
        "got webp image data from url : {:.4}sec",
        now.elapsed().as_millis() as f64 / 1000.0
    );

    reader_buf.rewind().unwrap();
    println!(
        "rewinded buf : {:.4}",
        now.elapsed().as_millis() as f64 / 1000.0
    );

    let decoded_webp = WebPDecoder::new(reader_buf).unwrap();
    let mut result_buf = BufWriter::new(Vec::new());

    //default extension
    let extension = if decoded_webp.has_animation() {
        println!(
            "generated WebPDecoder: {:.4}sec",
            now.elapsed().as_millis() as f64 / 1000.0
        );
        println!("transfering webp to gif..");
        let frames = decoded_webp.into_frames();
        println!(
            "transfered webps to frames: {:.4}sec",
            now.elapsed().as_millis() as f64 / 1000.0
        );
        let mut encoding_gif = GifEncoder::new_with_speed(&mut result_buf, 10);
        println!(
            "generated gif encoder: {:.4}sec",
            now.elapsed().as_millis() as f64 / 1000.0
        );
        encoding_gif.try_encode_frames(frames).unwrap();
        println!(
            "finished encoding frames: {:.4}sec",
            now.elapsed().as_millis() as f64 / 1000.0
        );
        encoding_gif.set_repeat(Repeat::Infinite).unwrap();
        println!(
            "transfered webp to gif. {:.4}sec",
            now.elapsed().as_millis() as f64 / 1000.0
        );
        ".gif"
    } else {
        let (result_width, result_height) = decoded_webp.dimensions();
        let mut read_image = vec![0; decoded_webp.total_bytes() as usize];
        decoded_webp.read_image(&mut read_image).unwrap();

        PngEncoder::new(&mut result_buf)
            .write_image(
                &read_image[..],
                result_width,
                result_height,
                ColorType::Rgba8,
            )
            .unwrap();
        println!("transfered webp to png.");
        ".png"
    };

    CreateAttachment::bytes(
        result_buf.into_inner().unwrap().to_vec(),
        "transfered".to_string() + extension,
    )
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

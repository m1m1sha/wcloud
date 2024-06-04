use image::{ImageFormat, Rgba};
use nanorand::{Rng, WyRand};
use palette::{Hsl, IntoColor, Pixel, Srgb};
use std::collections::HashSet;
use wcloud::{Tokenizer, Word, WordCloud, WordCloudSize, DEFAULT_EXCLUDE_WORDS_TEXT};

use std::time::Instant;

fn main() {
    let script_text =
        include_str!("a_new_hope.txt").replace("HAN", "Han").replace("LUKE'S", "Luke");

    let mut filter = DEFAULT_EXCLUDE_WORDS_TEXT.lines().collect::<HashSet<_>>();

    filter.insert("int");
    filter.insert("ext");

    let tokenizer = Tokenizer::default().with_max_words(1000).with_filter(filter);

    let word_cloud =
        WordCloud::default().with_tokenizer(tokenizer).with_word_margin(10).with_rng_seed(1);

    let mask_buf = include_bytes!("stormtrooper_mask.png");
    let mask_image = image::load_from_memory_with_format(mask_buf, ImageFormat::Png)
        .expect("Unable to load mask from memory")
        .to_luma8();

    let mask = WordCloudSize::FromMask(mask_image);

    let color_func = |_word: &Word, rng: &mut WyRand| {
        let lightness = rng.generate_range(40..100);

        let col = Hsl::new(0.0, 0.0, lightness as f32 / 100.0);
        let rgb: Srgb = col.into_color();

        let raw: [u8; 3] = rgb.into_format().into_raw();

        Rgba([raw[0], raw[1], raw[2], 1])
    };

    let now = Instant::now();
    let word_cloud_image =
        word_cloud.generate_from_text_with_color_func(&script_text, mask, 1.0, color_func);

    println!("Generated in {}ms", now.elapsed().as_millis());

    word_cloud_image.save("examples/custom_colors/a_new_hope.png").expect("Unable to save image");
}

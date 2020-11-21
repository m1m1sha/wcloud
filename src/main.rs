use regex::Regex;
use wcloud::{Tokenizer, Word, WordCloud, WordCloudSize};

use image::{DynamicImage, Rgba, GenericImage, GenericImageView, GrayImage, Luma, Rgb, RgbImage};

mod text;

use std::collections::HashSet;

fn main() {
    let text = "Of course it was a disaster.
That unbearable, dearest secret
has always been a disaster.
The danger when we try to leave.
Going over and over afterward
what we should have done
instead of what we did.
But for those short times
we seemed to be alive. Misled,
misused, lied to and cheated,
certainly. Still, for that
little while, we visited
our possible life.";
    // let exclude_words: HashSet<&str> = vec!["we"].into_iter().collect();

    let mask_path = "masks/joshmask.png";
    let mut mask_image = image::open(mask_path).unwrap().to_luma();

    let wordcloud_size = WordCloudSize::FromDimensions { width: 800, height: 400 };
    // let wordcloud_size = WordCloudSize::FromMask(mask_image);
    let wordcloud = WordCloud::default();
    let wordcloud = wordcloud.generate_from_text(text, wordcloud_size);

    wordcloud.save("output.png")
        .expect("Failed to save WordCloud image");
}

use ab_glyph::FontVec;
use clap::{arg, command, Parser};
use csscolorparser::Color;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder, Rgba};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{self, stdout, Read};
use wcloud::{Tokenizer, WordCloud, WordCloudSize, DEFAULT_EXCLUDE_WORDS_TEXT};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specifies the file of words to build the word cloud with
    #[arg(short, long)]
    text: Option<String>,

    /// Sets a custom regex to tokenize words with
    #[arg(long)]
    regex: Option<String>,

    /// Sets the width of the word cloud
    #[arg(long, default_value_t = 400)]
    width: u32,

    /// Sets the height of the word cloud
    #[arg(long, default_value_t = 200)]
    height: u32,

    /// Sets the scale of the final word cloud image, relative to the width and height
    #[arg(long, default_value_t = 1.0)]
    scale: f32,

    /// Sets the background color of the word cloud image
    #[arg(long)]
    background: Option<String>,

    /// Sets the spacing between words
    #[arg(long)]
    margin: Option<u32>,

    /// Sets the maximum number of words to display in the word cloud
    #[arg(long)]
    max_words: Option<u32>,

    /// Sets the minimum font size for words
    #[arg(long)]
    min_font_size: Option<f32>,

    /// Sets the maximum font size for words
    #[arg(long)]
    max_font_size: Option<f32>,

    /// Sets the randomness seed for the word cloud for reproducible word clouds
    #[arg(long)]
    random_seed: Option<u64>,

    /// Whether to repeat words until the maximum word count is reached
    #[arg(long, default_value_t = false)]
    repeat: bool,

    /// Sets the amount to decrease the font size by when no space can be found for a word
    #[arg(long)]
    font_step: Option<f32>,

    /// Sets the chance that words are rotated (0.0 - not at all, 1.0 - every time) [0.1]
    #[arg(long)]
    rotate_chance: Option<f64>,

    /// Sets how much of an impact word frequency has on the font size of the word (0.0 - 1.0) [0.5]
    #[arg(long)]
    relative_scaling: Option<f32>,

    /// Sets the boolean mask image for the word cloud shape. Any color other than black (#000) means there is no space
    #[arg(long)]
    mask: Option<String>,

    /// A newline-separated list of words to exclude from the word cloud
    #[arg(long)]
    exclude_words: Option<String>,

    /// Sets the output file for the word cloud image
    #[arg(short, long)]
    output: Option<String>,

    /// Sets the font used for the word cloud
    #[arg(short, long)]
    font: Option<String>,

    /// Sets the output format for the word cloud image (png, svg)
    #[arg(long)]
    format: Option<String>,
}

fn main() {
    let args = Args::parse();
    let mut tokenizer = Tokenizer::default();

    tokenizer = tokenizer.with_repeat(args.repeat);

    if let Some(max_words) = args.max_words {
        tokenizer = tokenizer.with_max_words(max_words);
    }

    if let Some(regex_str) = args.regex {
        let regex = match Regex::new(&regex_str) {
            Ok(regex) => regex,
            Err(e) => {
                println!("{}", e);
                std::process::exit(1)
            }
        };
        tokenizer = tokenizer.with_regex(regex);
    }

    let exclude_words = if let Some(exclude_words_path) = args.exclude_words {
        fs::read_to_string(exclude_words_path.clone()).unwrap_or_else(|_| {
            panic!("Unable to read exclude words file \'{}\'", exclude_words_path)
        })
    } else {
        // Default exclude list taken from the WordCloud for Python project
        // https://github.com/amueller/word_cloud/blob/master/word_cloud/stopwords
        DEFAULT_EXCLUDE_WORDS_TEXT.to_string()
    };

    if !exclude_words.is_empty() {
        let exclude_words = exclude_words.lines().collect::<HashSet<_>>();
        tokenizer = tokenizer.with_filter(exclude_words);
    }

    let word_cloud_size = match args.mask {
        Some(mask_path) => {
            let mask_image = image::open(mask_path).unwrap().into_luma8();

            WordCloudSize::FromMask(mask_image)
        }
        None => WordCloudSize::FromDimensions { width: args.width, height: args.height },
    };

    let background_color = match args.background {
        Some(color) => {
            let col = color.parse::<Color>().unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0)).to_rgba8();

            Rgba(col)
        }
        None => Rgba([0, 0, 0, 0]),
    };

    let mut word_cloud =
        WordCloud::default().with_tokenizer(tokenizer).with_background_color(background_color);

    if let Some(margin) = args.margin {
        word_cloud = word_cloud.with_word_margin(margin);
    }

    if let Some(min_font_size) = args.min_font_size {
        word_cloud = word_cloud.with_min_font_size(min_font_size);
    }

    if let Some(max_font_size) = args.max_font_size {
        word_cloud = word_cloud.with_max_font_size(Some(max_font_size));
    }

    if let Some(random_seed) = args.random_seed {
        word_cloud = word_cloud.with_rng_seed(random_seed);
    }

    if let Some(font_step) = args.font_step {
        word_cloud = word_cloud.with_font_step(font_step);
    }

    if let Some(rotate_chance) = args.rotate_chance {
        word_cloud = word_cloud.with_word_rotate_chance(rotate_chance);
    }

    if let Some(font_path) = args.font {
        let font_file = fs::read(font_path).expect("Unable to read font file");

        word_cloud = word_cloud
            .with_font(FontVec::try_from_vec(font_file).expect("Font file may be invalid"));
    }

    let text = if let Some(text_file_path) = args.text {
        fs::read_to_string(text_file_path.clone())
            .unwrap_or_else(|_| panic!("Unable to read text file \'{}\'", text_file_path))
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Unable to read stdin");
        buffer
    };

    let word_cloud_image = word_cloud.generate_from_text(&text, word_cloud_size, args.scale);

    if let Some(file_path) = args.output {
        word_cloud_image.save(file_path).expect("Failed to save WordCloud image");
    } else {
        // TODO: support SVG output
        let encoder = PngEncoder::new(stdout());

        let width = word_cloud_image.width();
        let height = word_cloud_image.height();

        encoder
            .write_image(&word_cloud_image, width, height, ColorType::Rgb8.into())
            .expect("Failed to save word_cloud image");
    }
}

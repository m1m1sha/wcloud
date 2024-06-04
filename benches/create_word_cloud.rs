use image::Rgba;
use nanorand::{Rng, WyRand};
use palette::{Hsl, IntoColor, Pixel, Srgb};
use std::collections::HashSet;
use std::fs;
use wcloud::{sat, Tokenizer, Word, WordCloud, WordCloudSize, DEFAULT_EXCLUDE_WORDS_TEXT};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

pub fn wcloud(c: &mut Criterion) {
    let mut group = c.benchmark_group("create star wars");
    group.sample_size(10);

    let script_path = "examples/custom_colors/a_new_hope.txt";
    let script_text = fs::read_to_string(script_path)
        .expect("Unable to find a_new_hope.txt")
        .replace("HAN", "Han")
        .replace("LUKE'S", "Luke");

    // word_cloud_image.save("examples/custom_colors/a_new_hope.png")
    //     .expect("Unable to save image to examples/a_new_hope.png");

    group.bench_function("generate word cloud", |b| {
        b.iter(|| {
            let mut filter = DEFAULT_EXCLUDE_WORDS_TEXT.lines().collect::<HashSet<_>>();

            filter.insert("int");
            filter.insert("ext");

            let tokenizer = Tokenizer::default().with_max_words(1000).with_filter(filter);

            let word_cloud = WordCloud::default()
                .with_tokenizer(tokenizer)
                .with_word_margin(10)
                .with_rng_seed(1);

            let mask_path = "examples/custom_colors/stormtrooper_mask.png";
            let mask_image = image::open(mask_path).unwrap().to_luma8();
            let mask = WordCloudSize::FromMask(mask_image);

            let color_func = |_word: &Word, rng: &mut WyRand| {
                let lightness = rng.generate_range(40..100);

                let col = Hsl::new(0.0, 0.0, lightness as f32 / 100.0);
                let rgb: Srgb = col.into_color();

                let raw: [u8; 3] = rgb.into_format().into_raw();

                Rgba([raw[0], raw[1], raw[2], 1])
            };

            word_cloud.generate_from_text_with_color_func(
                &script_text,
                mask,
                black_box(1.0),
                color_func,
            )
        })
    });

    group.finish();
}

pub fn sat(c: &mut Criterion) {
    let mut group = c.benchmark_group("summed area table");
    group.sample_size(10);

    let table_sizes =
        [(10, 10), (100, 100), (1000, 1000), (1920, 1080), (2560, 1440), (3840, 2160)];
    let mut rng = WyRand::new();

    for size in table_sizes {
        group.throughput(Throughput::Bytes(size.0 * size.1));
        group.bench_with_input(
            BenchmarkId::from_parameter(size.0 * size.1),
            &(size.0 * size.1),
            |b, &table_len| {
                let mut table: Vec<u32> =
                    (0..table_len).map(|_| rng.generate_range(0_u32..=255)).collect();
                b.iter(|| sat::to_summed_area_table(&mut table, size.0 as usize, 0));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, wcloud, sat,);
criterion_main!(benches);

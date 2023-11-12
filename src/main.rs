use std::fs;

use clap::Parser;
use colored::*;
use image::{ImageBuffer, Rgba};
use serde::Serialize;

use kmeans_colors::{get_kmeans_hamerly, Sort};
use palette::cast::ComponentsAs;
use palette::luma::Luma;
use palette::rgb::Rgb;
use palette::{named, FromColor, IntoColor, Srgb, Srgba};

use tinytemplate::TinyTemplate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    image: String,
}

#[derive(Serialize)]
struct Context {
    background: String,
    foreground: String,
    cursor: String,
    color0: String,
    color1: String,
    color2: String,
    color3: String,
    color4: String,
    color5: String,
    color6: String,
    color7: String,
    color8: String,
    color9: String,
    color10: String,
    color11: String,
    color12: String,
    color13: String,
    color14: String,
    color15: String,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            background: "#000000".to_string(),
            foreground: "#FFFFFF".to_string(),
            cursor: "#FFFFFF".to_string(),
            color0: "#FFFFFF".to_string(),
            color1: "#FFFFFF".to_string(),
            color2: "#FFFFFF".to_string(),
            color3: "#FFFFFF".to_string(),
            color4: "#FFFFFF".to_string(),
            color5: "#FFFFFF".to_string(),
            color6: "#FFFFFF".to_string(),
            color7: "#FFFFFF".to_string(),
            color8: "#FFFFFF".to_string(),
            color9: "#FFFFFF".to_string(),
            color10: "#FFFFFF".to_string(),
            color11: "#FFFFFF".to_string(),
            color12: "#FFFFFF".to_string(),
            color13: "#FFFFFF".to_string(),
            color14: "#FFFFFF".to_string(),
            color15: "#FFFFFF".to_string(),
        }
    }
}

fn main() {
    let args = Args::parse();
    let img = image::open(args.image).expect("Failed to open image");
    let thumb = img.thumbnail(400, 400).into_rgba8();
    // let thumb = img.into_rgba8();

    extract_colors(&thumb, 16);
}

fn print_palette(colors: Vec<Srgb>) {
    for color in colors.iter() {
        let col = color.into_format::<u8>();
        let color_block = "   ".on_truecolor(col.red, col.green, col.blue);
        print!("{}", color_block);
    }

    println!();

    let color_strings: Vec<String> = colors
        .iter()
        .map(|c| format!("#{:x}", c.into_format::<u8>()))
        .collect();

    println!("COLORS: {:?}", color_strings);
}

fn print_color(color: Srgb) {
    let col = color.into_format::<u8>();
    let color_block = "   ".on_truecolor(col.red, col.green, col.blue);
    println!("{}", color_block);
    println!("#{:x}", col);
}

fn extract_colors(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, num_colors: usize) {
    // TODO: maybe allow for custom seed?
    let seed = 0;

    let img_vec: &[Srgba<u8>] = img.as_raw().components_as();
    let converge = 0.0025;

    // Vec of pixels converted to Srgb<f32>; cleared and reused between runs
    let rgb_pixels: Vec<Srgb<f32>> = img_vec
        .iter()
        .filter(|x| x.alpha == 255)
        .map(|x| Srgb::<f32>::from_color(x.into_format::<_, f32>()))
        .collect();

    let result = get_kmeans_hamerly(num_colors, 20, converge, false, &rgb_pixels, seed);
    let mut res = Srgb::sort_indexed_colors(&result.centroids, &result.indices);
    res.sort_unstable_by(|a, b| (b.percentage).total_cmp(&a.percentage));

    let colors: Vec<Srgb> = res.iter().map(|c| c.centroid.into_color()).collect();

    let darkest_color = colors
        .iter()
        .min_by(|&color_a, &color_b| {
            let luma_a: Luma = Luma::from_color(*color_a);
            let luma_b: Luma = Luma::from_color(*color_b);
            luma_a
                .into_components()
                .partial_cmp(&luma_b.into_components())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(Rgb::from_format(named::BLACK), |c| *c);

    let lightest_color = colors
        .iter()
        .max_by(|&color_a, &color_b| {
            let luma_a: Luma = Luma::from_color(*color_a);
            let luma_b: Luma = Luma::from_color(*color_b);
            luma_a
                .into_components()
                .partial_cmp(&luma_b.into_components())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(Rgb::from_format(named::WHITE), |c| *c);

    print!("DARKEST COLOR ");
    print_color(darkest_color);
    let background = format!("#{:x}", darkest_color.into_format::<u8>());

    print!("LIGHTEST COLOR ");
    print_color(lightest_color);

    print_palette(colors);

    let template = fs::read_to_string("./templates/gtk.css").unwrap();

    let mut tt = TinyTemplate::new();
    tt.add_template("default", &template).unwrap();

    let context = Context {
        background,
        ..Context::default()
    };

    let rendered = tt.render("default", &context).unwrap();

    println!("{}", rendered);
}

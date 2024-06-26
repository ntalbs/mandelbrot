use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, ImageError};
use num::Complex;
use rayon::prelude::*;
use std::{fs::File, str::FromStr};

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn parse_width<T: FromStr>(s: &str) -> Option<T> {
    match T::from_str(s) {
        Ok(v) => Some(v),
        _ => None,
    }
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    top_left: Complex<f64>,
    bottom_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (bottom_right.re - top_left.re, top_left.im - bottom_right.im);
    Complex {
        re: top_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: top_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    top_left: Complex<f64>,
    bottom_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), top_left, bottom_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            }
        }
    }
}

fn write_image(fliename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), ImageError> {
    let output = File::create(fliename)?;
    let encoder = PngEncoder::new(output);
    encoder.write_image(
        pixels,
        bounds.0 as u32,
        bounds.1 as u32,
        ColorType::L8.into(),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS_WIDTH RE_RANGE IM_RANGE", args[0]);
        eprintln!("Example: {} mandel.png 2000 -1.20,0.35 -1,0.20", args[0]);
        std::process::exit(1);
    }

    let width_px = parse_width(&args[2]).expect("error parsing image width");
    let (left, right) = parse_pair(&args[3], ',').expect("error parsing re range");
    let (bottom, top) = parse_pair(&args[4], ',').expect("error parsing im range");

    let top_left = Complex { re: left, im: top };
    let bottom_right = Complex {
        re: right,
        im: bottom,
    };

    let width = bottom_right.re - top_left.re;
    let height = top_left.im - bottom_right.im;

    let ratio = width_px as f64 / width;
    let height_px = (height * ratio) as usize;
    let bounds = (width_px, height_px);

    let mut pixels = vec![0; bounds.0 * bounds.1];
    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds = (bounds.0, 1);
        let band_top_left = pixel_to_point(bounds, (0, top), top_left, bottom_right);
        let band_bottom_right = pixel_to_point(bounds, (bounds.0, top + 1), top_left, bottom_right);
        render(band, band_bounds, band_top_left, band_bottom_right);
    });
    write_image(&args[1], &pixels, bounds).expect("error writing PNG file");
}

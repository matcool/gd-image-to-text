use std::{collections::HashMap, io::Write};

use flate2::{write::GzEncoder, Compression};
use image::{buffer::ConvertBuffer, imageops::FilterType, GenericImageView, Pixel, Rgb, RgbImage};

fn floyd_steinberg_dither<C: Fn(Rgb<u8>) -> Rgb<u8>>(img: &mut RgbImage, color_pred: C) {
	for y in 0..img.height() {
		for x in 0..img.width() {
			let old_color = *img.get_pixel(x, y);
			let new_color = color_pred(old_color);
			img.put_pixel(x, y, new_color);
			let error: [i32; 3] = old_color.0.map(|x| x as i32);
			let error: [i32; 3] = [
				error[0] - new_color[0] as i32,
				error[1] - new_color[1] as i32,
				error[2] - new_color[2] as i32,
			];
			const FACTORS: [f32; 4] = [7.0, 5.0, 3.0, 1.0];
			const OFFSETS: [(i32, i32); 4] = [(1, 0), (-1, 1), (0, 1), (1, 1)];
			for ((ox, oy), factor) in OFFSETS.into_iter().zip(FACTORS) {
				if x == 0 && ox < 0 {
					continue;
				}
				let x = (x as i32 + ox) as u32;
				let y = (y as i32 + oy) as u32;
				if !img.in_bounds(x, y) {
					continue;
				}
				let mut color = *img.get_pixel(x, y);
				for (col, err) in color.0.iter_mut().zip(error) {
					*col = (*col as f32 + (err as f32 * factor / 16.0)).clamp(0.0, 255.0) as u8;
				}
				img.put_pixel(x, y, color);
			}
		}
	}
}

// O  5094 1.00
// o  4181 0.82
// =  1292 0.25
// .. 972  0.19
//    0    0.00

fn show_help() -> ! {
	println!(
		"Geometry Dash image to text conversion tool

Usage:
    gd-image-to-text [OPTIONS] <path>

Args:
    <path>                  Path for the input image

Options:
    -g, --grayscale         Turns the image grayscale, making it only use one text object
    -o, --output <path>     Specify path for output .gmd file, if not specified will
                            open a save file dialog instead
    -s, --size <size>       Size for the image in *characters*, given in \"WxH\" format,
                            if not specified will use the image's actual width and height.
                            Keep in mind GD characters are about twice as tall as their width
    --scale <scale>         Scale for the text objects, defaults to 0.075
"
	);
	std::process::exit(1)
}

fn main() {
	if std::env::args().len() < 2 {
		show_help();
	}

	let mut path = None;
	let mut grayscale = false;
	let mut output_path = None;
	let mut args_iter = std::env::args().skip(1);
	let mut user_size: Option<(u32, u32)> = None;
	let mut object_scale = 0.075;
	while let Some(arg) = args_iter.next() {
		match arg.as_str() {
			"-g" | "--grayscale" => {
				grayscale = true;
			}
			"-o" | "--output" => {
				output_path = args_iter.next().map(std::path::PathBuf::from);
				if output_path.is_none() {
					eprintln!("err: Expected output path to --output");
					show_help();
				}
			}
			"-s" | "--size" => {
				let arg = args_iter.next();
				if arg.is_none() {
					eprintln!("err: Expected width and height to --size");
					show_help()
				}
				let arg = arg.unwrap();
				let split = arg.split_once('x');
				if split.is_none() {
					eprintln!("err: Size must be in \"WxH\" format");
					show_help();
				}
				let split = split.unwrap();
				let check = |_| {
					eprintln!("err: Invalid size");
					show_help();
				};
				let width = split.0.parse::<u32>().unwrap_or_else(check);
				let height = split.1.parse::<u32>().unwrap_or_else(check);
				user_size = Some((width, height));
			}
			"--scale" => {
				let arg = args_iter.next();
				if arg.is_none() {
					eprintln!("err: Expected scale");
					show_help()
				}
				object_scale = arg.unwrap().parse().unwrap_or_else(|_| {
					eprintln!("err: Invalid scale");
					show_help()
				});
			}
			_ => {
				if path.is_some() {
					eprintln!("err: Unknown extra argument");
					show_help();
				}
				path = Some(arg.clone());
			}
		}
	}
	if path.is_none() {
		eprintln!("err: Expected path");
		show_help();
	}
	let path = path.unwrap();

	let mut img = image::open(&path)
		.unwrap_or_else(|_| {
			eprintln!("err: Could not open image: {path}");
			std::process::exit(1)
		})
		.to_rgba8();
	// multiply alpha into color
	// essentially adding a black background to transparent images
	img.pixels_mut().for_each(|pixel| {
		let alpha = pixel.0[3] as u16;
		pixel.apply_without_alpha(|x| ((x as u16 * alpha) / 255) as u8);
	});
	let img = image::DynamicImage::ImageRgb8(img.convert());

	let characters = HashMap::from([
		((1.00 * 255.0) as u8, "O"),
		((0.82 * 255.0) as u8, "o"),
		((0.25 * 255.0) as u8, "="),
		((0.19 * 255.0) as u8, ".."),
		((0.00 * 255.0) as u8, "  "),
	]);

	let closest_value = |x: u8| {
		*characters
			.keys()
			.min_by_key(|&&value| (value as i32 - x as i32).abs())
			.unwrap() // iterator should never be empty
	};

	// limit of characters a single batch node can render
	// any more and they wont render
	const LIMIT: usize = 16384;

	let process_image = |mut img: RgbImage| {
		floyd_steinberg_dither(&mut img, |color| color.map(closest_value));

		let mut layers: [String; 3] = ["".into(), "".into(), "".into()];

		let mut max_width = [0usize; 3];

		let calculate_line_width = |line: &str| {
			// change these if messing with the font :-)
			let doubles = line.matches(|c| c == '.' || c == ' ').count();
			line.len() - doubles / 2
		};

		for row in img.rows() {
			let mut lines: [String; 3] = ["".into(), "".into(), "".into()];
			for pixel in row {
				if grayscale {
					let avg = pixel.channels().iter().map(|x| *x as u32).sum::<u32>() / 3;
					lines[0].push_str(characters.get(&closest_value(avg as u8)).unwrap());
				} else {
					for (i, pix) in pixel.channels().iter().enumerate() {
						lines[i].push_str(characters.get(pix).unwrap());
					}
				}
			}
			for (i, line) in lines.iter().enumerate() {
				let trimmed = line.trim_end_matches(' ');
				max_width[i] = max_width[i].max(calculate_line_width(trimmed));
				layers[i].push_str(if trimmed.is_empty() { " " } else { trimmed });
				layers[i].push('\n');
			}
		}

		for (i, layer) in layers.iter_mut().enumerate() {
			layer.pop(); // remove trailing \n

			// if the max width wasnt big enough add trailing spaces
			// to the first line
			if max_width[i] != img.width() as usize {
				let line_end = layer.find('\n').unwrap();
				let line = &layer[0..line_end];
				let width = calculate_line_width(line) as u32;
				for _ in 0..(img.width() - width) {
					layer.insert_str(line_end, characters.get(&0).unwrap());
				}
			}
		}

		layers
	};

	let mut layers;
	let mut size = user_size.unwrap_or_else(|| img.dimensions());
	if user_size.is_none() {
		// characters have an aspect ratio of about 1:2
		// so do this to keep aspect ratio in gd close to original image
		size.1 /= 2;
	}
	loop {
		layers = process_image(
			img.resize_exact(size.0, size.1, FilterType::Nearest)
				.to_rgb8(),
		);
		let max = layers.iter().map(|layer| layer.len()).max().unwrap();
		println!("char count is {}", max);
		if max > LIMIT {
			let factor = (LIMIT as f32 / max as f32).sqrt();
			size = (
				(size.0 as f32 * factor) as u32,
				(size.1 as f32 * factor) as u32,
			);
			println!("too big, trying with size {size:?}");
		} else {
			break;
		}
	}

	// base of the output level
	// contains:
	// font 10, most importantly
	// color 1 = pure white with blending
	// color 2 = pure red with blending
	// color 3 = pure green with blending
	// color 4 = pure blue with blending
	// 0.5x speed mini ship
	let mut level_string = String::from("kS38,1_255_2_0_3_0_4_-1_5_1_6_2_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_125_2_255_3_0_4_-1_5_0_6_1005_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_0_3_0_4_-1_5_0_6_1000_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_255_2_255_3_255_4_-1_5_0_6_1002_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_255_3_0_4_-1_5_1_6_3_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_102_3_255_4_-1_5_0_6_1009_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_255_2_255_3_255_4_-1_5_1_6_1_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_0_3_0_4_-1_5_0_6_1001_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_255_3_255_4_-1_5_0_6_1006_7_1_8_1_11_255_12_255_13_255_15_1_18_0|1_0_2_0_3_255_4_-1_5_1_6_4_7_1_8_1_11_255_12_255_13_255_15_1_18_0,kA2,1,kA3,1,kA4,1,kA6,0,kA7,0,kA8,0,kA9,0,kA10,0,kA11,0,kA13,0,kA15,0,kA16,0,kA17,0,kA18,9,kS39,0;1,899,2,0,3,165,7,255,8,255,9,255,10,0,17,1;1,899,2,0,3,135,7,255,8,0,9,0,23,2,10,0,17,1;1,899,2,0,3,105,7,0,8,255,9,0,23,3,10,0,17,1;1,899,2,0,3,75,7,0,8,0,9,255,23,4,10,0,17,1");

	for (i, layer) in layers.iter().enumerate() {
		// only care about first layer on grayscale mode
		if grayscale && i != 0 {
			continue;
		}

		let b64_text = base64::encode_config(layer, base64::URL_SAFE);
		let x = 285.0;
		let y = 150.0;
		let z_layer = i as i32 * 2 - 3; // uses layers B4, B3 and B2
		let color_1 = if grayscale { 1 } else { i + 2 };
		level_string.push_str(
			format!(";1,914,2,{x},3,{y},31,{b64_text},32,{object_scale},21,{color_1},24,{z_layer}")
				.as_str(),
		);
	}

	let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
	encoder.write_all(level_string.as_bytes()).unwrap();
	let level_string = base64::encode_config(encoder.finish().unwrap(), base64::URL_SAFE);

	let description =
		base64::encode_config("Generated using gd-image-to-text 1.0.0", base64::URL_SAFE);
	// base64 it twice, because hjfod
	let description = base64::encode_config(description, base64::URL_SAFE);

	if output_path.is_none() {
		if let Some(path) = rfd::FileDialog::new()
			.add_filter("GMD File", &["gmd"])
			.set_title("Save .gmd file")
			.save_file()
		{
			output_path = Some(path);
		} else {
			println!("no path :(");
			std::process::exit(1);
		}
	}

	std::fs::write(
		output_path.unwrap(),
		format!(
			"<d>
	<k>kCEK</k><i>4</i>
	<k>k2</k><s>Image to text level</s>
	<k>k3</k><s>{description}</s>
	<k>k4</k><s>{level_string}</s>
	<k>k13</k><t/>
	<k>k21</k><i>2</i>
	<k>k50</k><i>35</i>
</d>"
		),
	)
	.unwrap();
}

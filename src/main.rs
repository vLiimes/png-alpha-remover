use std::env::{current_dir, current_exe};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::io::BufWriter;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    remove_alpha: bool,

    // Flips it horizontally (So ACROSS the vertical axis)
    #[arg(short, long)]
    horizontal_flip: bool,

    // Flips it vertically (So ACROSS the horizontal axis)
    #[arg(short, long)]
    vertical_flip: bool,
}


fn main() {
    let directory = fs::read_dir(".").unwrap();
    let args = Args::parse();
    
    for file in directory {
        if let Ok(file) = file {
            if let Ok(file_type) = file.file_type() {
                if file_type.is_dir() {
                    continue;
                }

                let file_path = file.path();

                let exten = match file_path.extension() {
                    Some(extension) => extension,
                    None => { continue; }
                };

                let lower = exten.to_ascii_lowercase();

                let exten_str = lower.to_str().unwrap();

                if exten_str != "png" { continue; }

                if args.remove_alpha {
                    remove_alpha_add_background(&file_path);
                }

                if args.horizontal_flip {

                }

                if args.vertical_flip {

                }
            }
        }
    }
}

fn remove_alpha_add_background(png_file_path: &PathBuf) {
    let decoder: png::Decoder<BufReader<File>> = png::Decoder::new(BufReader::new(File::open(png_file_path).unwrap()));

    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image.
    let mut bytes = &mut buf[..info.buffer_size()];

    for i in 0..bytes.len() {
        if (i + 1) % 4 != 0 { continue; }

        let top = [bytes[i - 3] as f64/255.0, bytes[i - 2] as f64/255.0, bytes[i - 1] as f64/255.0];

        let alpha = bytes[i] as f64 / 255.0;

        let result = [1.0 - alpha + (top[0] * alpha), 1.0 - alpha + (top[1] * alpha),  1.0 - alpha + (top[2] * alpha)];

        for j in 0..3 {
            if result[j] > 1.0 || result[j] < 0.0 {
                panic!("Ratio outside of allowed values!");
            }
        }

        let rgb_vals = result.map(|val| {(val * 255.0).round() as u8});

        for j in 0..3 {
            bytes[i - (3 - j)] = rgb_vals[j];
        }

        bytes[i] = 255;
    }

    let input_file_stem = (png_file_path).file_stem().unwrap().to_str().unwrap();

    let output_file_name = format!("{}_no_alpha.png", input_file_stem);

    save_from_bytes(png_file_path, bytes, output_file_name, String::from("./alpha_removed/"));
}


fn horizontal_flip(png_file_path: &PathBuf) {
    let mut decoder = png::Decoder::new(BufReader::new(File::open(png_file_path).unwrap()));
    let info = decoder.read_header_info().unwrap();

    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image.
    let mut bytes = &mut buf[..info.buffer_size()];

    let mut row_start: usize = 0;
    let width = info.width as usize;
    let width_limit = (width / 2) - 1;

    for i in 0..(info.height as usize) {
        for j in 0..width_limit {
            let right_val = bytes[row_start + width - (j + 1)];

            bytes[row_start + width - (j + 1)] = bytes[row_start + j];
            bytes[row_start + j] = right_val;
        }

        row_start = width * i;
    }

    let input_file_stem = (png_file_path).file_stem().unwrap().to_str().unwrap();

    let output_file_name = format!("{}_horizontal_flip.png", input_file_stem);

    save_from_bytes(png_file_path, bytes, output_file_name, String::from("./horizontal_flip/"));
}

fn vertical_flip(png_file_path: &PathBuf) {
    let mut decoder = png::Decoder::new(BufReader::new(File::open(png_file_path).unwrap()));
    let info = decoder.read_header_info().unwrap();

    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image.
    let mut bytes = &mut buf[..info.buffer_size()];

    let width = info.width as usize;
    let height = info.height as usize;
    let height_limit = (height / 2) - 1;

    for i in 0..width {
        for j in 0..height_limit {
            let bottom_val = bytes[i + (width * (height - j))]; 

            bytes[i + (width * (height - j))] = bytes[i + (width * j)];
            
            bytes[i + (width * j)] = bottom_val;
        }
    }

    let input_file_stem = (png_file_path).file_stem().unwrap().to_str().unwrap();

    let output_file_name = format!("{}_vertical_flip.png", input_file_stem);

    save_from_bytes(png_file_path, bytes, output_file_name, String::from("./vertical_flip/"));
}

fn save_from_bytes(png_file_path: &PathBuf, bytes: &[u8], output_file_name: String, output_dir: String) {
    

    let output_dir_file = format!("{}{}", &output_dir, &output_file_name);

    let output_path = Path::new(&output_dir_file);
    let file = File::create(output_path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut decoder_metadata = png::Decoder::new(BufReader::new(File::open(png_file_path).unwrap()));
    let input_metadata = (&mut decoder_metadata).read_header_info().unwrap();

    let mut encoder = png::Encoder::new(w, input_metadata.width, input_metadata.height); // Width is 2 pixels and height is 1.
    encoder.set_color(input_metadata.color_type);
    encoder.set_depth(input_metadata.bit_depth);

    match input_metadata.source_gamma {
        Some(val) => {
            encoder.set_source_gamma(val);
        }

        None => {
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        }
    }

    match input_metadata.source_chromaticities {
        Some(val) => {
            encoder.set_source_chromaticities(val);
        }

        None => {
            let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000)
            );
            encoder.set_source_chromaticities(source_chromaticities);
        }
    }
    let mut writer = encoder.write_header().unwrap();
    // An array containing a RGBA sequence. First pixel is red and second pixel is black.
    writer.write_image_data(&bytes).unwrap(); // Save
}

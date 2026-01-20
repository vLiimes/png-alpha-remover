use std::env::{current_dir, current_exe};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::io::BufWriter;


fn main() {
    let directory = fs::read_dir(".").unwrap();
    let output_directory = fs::create_dir("./alpha_removed").unwrap();

    
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

                let decoder = png::Decoder::new(BufReader::new(File::open(&file_path).unwrap()));

                let mut reader = decoder.read_info().unwrap();

                let mut buf = vec![0; reader.output_buffer_size().unwrap()];
                // Read the next frame. An APNG might contain multiple frames.
                let info = reader.next_frame(&mut buf).unwrap();
                // Grab the bytes of the image.
                let mut bytes = &mut buf[..info.buffer_size()];

                for i in 0..bytes.len() {
                    if (i + 1) % 4 != 0 { continue; }


                    // println!("Raw Vals: {}, {}, {}", bytes[i-3], bytes[i-2], bytes[i-1]);
                    let top = [bytes[i - 3] as f64/255.0, bytes[i - 2] as f64/255.0, bytes[i - 1] as f64/255.0];

                    
                    // println!("Ratios:");
                    // top.iter().for_each(|val| { println!("{val} "); });

                    let alpha = bytes[i] as f64 / 255.0;
                    // println!("Alpha: {alpha}");

                    let result = [1.0 - alpha + (top[0] * alpha), 1.0 - alpha + (top[1] * alpha),  1.0 - alpha + (top[2] * alpha)];

                    // println!("Blended:");
                    // result.iter().for_each(|val| { println!("{val} "); });

                    for j in 0..3 {
                        if result[j] > 1.0 || result[j] < 0.0 {
                            panic!("Ratio outside of allowed values!");
                        }
                    }

                    let rgb_vals = result.map(|val| {(val * 255.0).round() as u8});

                    // println!("Resulting vals:");
                    // rgb_vals.iter().for_each(|val| { println!("{val} "); });

                    for j in 0..3 {
                        bytes[i - (3 - j)] = rgb_vals[j];
                    }

                    bytes[i] = 255;
                }

                let input_file_stem = (&file_path).file_stem().unwrap().to_str().unwrap();

                let output_file_name = format!("{}_no_alpha.png", input_file_stem);

                let output_dir_file = format!("./alpha_removed/{}", &output_file_name);

                let output_path = Path::new(&output_dir_file);
                let file = File::create(output_path).unwrap();
                let ref mut w = BufWriter::new(file);

                let mut decoder_metadata = png::Decoder::new(BufReader::new(File::open(&file_path).unwrap()));
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
        }
    }
}

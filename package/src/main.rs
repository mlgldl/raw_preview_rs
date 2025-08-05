mod raw_converter;

use raw_converter::{convert_raw_to_jpeg, is_raw_file};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("RAW to JPEG Converter using LibRaw and libjpeg-turbo");
    println!("===================================");

    let test_raws_dir = "../test_raws";
    let output_dir = "../output";

    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    // Read the test RAW files directory
    let entries = match fs::read_dir(test_raws_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read test_raws directory: {}", e);
            return;
        }
    };

    // Process each RAW file found
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading directory entry: {}", e);
                continue;
            }
        };

        let input_path = entry.path();
        let file_name = input_path.file_name().unwrap().to_string_lossy();

        // Skip non-RAW files
        if !is_raw_file(&file_name) {
            continue;
        }

        // Generate output filename and path
        let input_path_str = input_path.to_string_lossy();
        let stem = input_path.file_stem().unwrap().to_string_lossy();
        let output_filename = format!("{}.jpg", stem);
        let output_path = Path::new(output_dir).join(&output_filename);
        let output_path_str = output_path.to_string_lossy();

        println!("Processing: {} -> {}", file_name, output_filename);

        // Convert RAW to JPEG and measure time
        let start_time = Instant::now();
        match convert_raw_to_jpeg(&input_path_str, &output_path_str) {
            Ok(()) => {
                let duration = start_time.elapsed();
                println!(
                    "  ✅ Success -> {} (took {:.2}s)",
                    output_filename,
                    duration.as_secs_f64()
                );
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("  ❌ Error: {} (took {:.2}s)", e, duration.as_secs_f64());
            }
        }
    }
}

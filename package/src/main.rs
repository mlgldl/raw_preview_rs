use raw_preview_rs::{get_file_type, is_supported_file, process_any_image};
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("Universal Image Processor using LibRaw and libjpeg-turbo");
    println!("=========================================================");

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

    // Process each image file found
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

        // Skip non-image files
        if !is_supported_file(&file_name) {
            continue;
        }

        // Generate output filename and path
        let input_path_str = input_path.to_string_lossy();
        let stem = input_path.file_stem().unwrap().to_string_lossy();
        let output_filename = format!("{}.jpg", stem);
        let output_path = Path::new(output_dir).join(&output_filename);
        let output_path_str = output_path.to_string_lossy();

        // Determine file type for display
        let file_type = get_file_type(&file_name);

        println!(
            "Processing {}: {} -> {}",
            file_type, file_name, output_filename
        );

        // Process image file and measure time
        let start_time = Instant::now();
        match process_any_image(&input_path_str, &output_path_str) {
            Ok(exif) => {
                let duration = start_time.elapsed();
                println!(
                    "  ✅ Success -> {} (took {:.2}s)",
                    output_filename,
                    duration.as_secs_f64()
                );
                println!("     📷 Camera: {} {}", exif.camera_make, exif.camera_model);
                if exif.has_exposure_info() {
                    if exif.iso_speed > 0 {
                        println!("     📊 ISO: {}", exif.iso_speed);
                    }
                    if exif.aperture > 0.0 {
                        println!("     🔍 Aperture: {}", exif.formatted_aperture());
                    }
                    if exif.shutter > 0.0 {
                        println!("     ⏱️  Shutter: {}", exif.formatted_shutter_speed());
                    }
                    if exif.focal_length > 0.0 {
                        println!("     📏 Focal Length: {:.0}mm", exif.focal_length);
                    }
                }
                let dimensions = exif.formatted_dimensions();
                if dimensions != "Unknown" {
                    println!("     📐 Image Size: {}", dimensions);
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("  ❌ Error: {} (took {:.2}s)", e, duration.as_secs_f64());
            }
        }
    }
}

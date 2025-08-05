use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;
use std::time::Instant;

// Foreign function interface to our C++ wrapper
unsafe extern "C" {
    fn process_raw_to_ppm(
        input_path: *const c_char,
        output_path: *const c_char,
        exif_data: *mut ExifData,
    ) -> i32;
    fn get_last_error() -> *const c_char;
}

#[repr(C)]
struct ExifData {
    camera_make: *const c_char,
    camera_model: *const c_char,
    software: *const c_char,
    iso_speed: i32,
    shutter: f64,
    aperture: f64,
    focal_length: f64,
    raw_width: i32,
    raw_height: i32,
    output_width: i32,
    output_height: i32,
    colors: i32,
    color_filter: i32,
    cam_mul: [f64; 4],
    date_taken: *const c_char,
    lens: *const c_char,
    max_aperture: f64,
    focal_length_35mm: i32,
    description: *const c_char,
    artist: *const c_char,
}

const RW_SUCCESS: i32 = 0;

fn process_raw_to_jpeg(input_path: &str, output_path: &str, quality: u8) -> Result<(), String> {
    let input_cstring =
        CString::new(input_path).map_err(|e| format!("Invalid input path: {}", e))?;
    let output_cstring =
        CString::new(output_path).map_err(|e| format!("Invalid output path: {}", e))?;

    let mut exif_data = ExifData {
        camera_make: ptr::null(),
        camera_model: ptr::null(),
        software: ptr::null(),
        iso_speed: 0,
        shutter: 0.0,
        aperture: 0.0,
        focal_length: 0.0,
        raw_width: 0,
        raw_height: 0,
        output_width: 0,
        output_height: 0,
        colors: 0,
        color_filter: 0,
        cam_mul: [0.0; 4],
        date_taken: ptr::null(),
        lens: ptr::null(),
        max_aperture: 0.0,
        focal_length_35mm: 0,
        description: ptr::null(),
        artist: ptr::null(),
    };

    let result = unsafe {
        process_raw_to_ppm(
            input_cstring.as_ptr(),
            output_cstring.as_ptr(),
            &mut exif_data,
        )
    };

    if result == RW_SUCCESS {
        Ok(())
    } else {
        let error_msg = unsafe {
            let error_ptr = get_last_error();
            if !error_ptr.is_null() {
                CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
            } else {
                "Unknown error".to_string()
            }
        };
        Err(format!("Error {}: {}", result, error_msg))
    }
}

fn main() {
    println!("RAW to JPEG Converter using LibRaw and libjpeg-turbo");
    println!("===================================");

    let test_raws_dir = "../test_raws";
    let output_dir = "../output";

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    let entries = match fs::read_dir(test_raws_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read test_raws directory: {}", e);
            return;
        }
    };

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

        let lower_name = file_name.to_lowercase();
        if !(lower_name.ends_with(".raw")
            || lower_name.ends_with(".cr2")
            || lower_name.ends_with(".cr3")
            || lower_name.ends_with(".nef")
            || lower_name.ends_with(".dng")
            || lower_name.ends_with(".arw")
            || lower_name.ends_with(".raf")
            || lower_name.ends_with(".rw2")
            || lower_name.ends_with(".orf"))
        {
            continue;
        }

        let input_path_str = input_path.to_string_lossy();
        let stem = input_path.file_stem().unwrap().to_string_lossy();
        let output_filename = format!("{}.jpg", stem);
        let output_path = Path::new(output_dir).join(&output_filename);
        let output_path_str = output_path.to_string_lossy();

        println!("Processing: {} -> {}", file_name, output_filename);

        let start_time = Instant::now();

        match process_raw_to_jpeg(&input_path_str, &output_path_str, 90) {
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

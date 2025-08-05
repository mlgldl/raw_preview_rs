use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

// Foreign function interface to our C++ wrapper
unsafe extern "C" {
    fn process_raw_to_jpeg(
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

/// Converts a RAW image file to JPEG format
///
/// # Arguments
/// * `input_path` - Path to the input RAW file
/// * `output_path` - Path where the output JPEG will be saved
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
///
/// # Example
/// ```
/// use raw_converter::convert_raw_to_jpeg;
///
/// match convert_raw_to_jpeg("photo.cr2", "photo.jpg") {
///     Ok(()) => println!("Conversion successful!"),
///     Err(e) => eprintln!("Conversion failed: {}", e),
/// }
/// ```
pub fn convert_raw_to_jpeg(input_path: &str, output_path: &str) -> Result<(), String> {
    let input_cstring =
        CString::new(input_path).map_err(|e| format!("Invalid input path: {}", e))?;
    let output_cstring =
        CString::new(output_path).map_err(|e| format!("Invalid output path: {}", e))?;

    // Initialize empty EXIF data structure (currently unused but required by API)
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

    // Call the C++ wrapper function
    let result = unsafe {
        process_raw_to_jpeg(
            input_cstring.as_ptr(),
            output_cstring.as_ptr(),
            &mut exif_data,
        )
    };

    if result == RW_SUCCESS {
        Ok(())
    } else {
        // Retrieve error message from C++ wrapper
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

/// Checks if a file extension corresponds to a supported RAW format
///
/// # Arguments
/// * `filename` - The filename to check
///
/// # Returns
/// * `true` if the file extension is a supported RAW format
/// * `false` otherwise
pub fn is_raw_file(filename: &str) -> bool {
    let lower_name = filename.to_lowercase();
    lower_name.ends_with(".raw")
        || lower_name.ends_with(".cr2")
        || lower_name.ends_with(".cr3")
        || lower_name.ends_with(".nef")
        || lower_name.ends_with(".dng")
        || lower_name.ends_with(".arw")
        || lower_name.ends_with(".raf")
        || lower_name.ends_with(".rw2")
        || lower_name.ends_with(".orf")
}

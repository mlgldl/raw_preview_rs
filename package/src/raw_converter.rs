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
    camera_make: [c_char; 64],
    camera_model: [c_char; 64],
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

/// Represents EXIF data extracted from a RAW file
#[derive(Debug, Clone)]
pub struct ExifInfo {
    pub camera_make: String,
    pub camera_model: String,
    pub software: String,
    pub iso_speed: i32,
    pub shutter: f64,
    pub aperture: f64,
    pub focal_length: f64,
    pub raw_width: i32,
    pub raw_height: i32,
    pub output_width: i32,
    pub output_height: i32,
    pub colors: i32,
    pub color_filter: i32,
    pub cam_mul: [f64; 4],
    pub date_taken: String,
    pub lens: String,
    pub max_aperture: f64,
    pub focal_length_35mm: i32,
    pub description: String,
    pub artist: String,
}

const RW_SUCCESS: i32 = 0;

/// Helper function to safely convert C string pointers to Rust strings
fn safe_string_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}

/// Helper function to safely convert C char arrays to Rust strings
fn safe_string_from_array(arr: &[c_char]) -> String {
    // Find the null terminator
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());

    // Convert to u8 slice and then to string
    let bytes: Vec<u8> = arr[..len].iter().map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Converts a RAW image file to JPEG format and extracts EXIF data
///
/// # Arguments
/// * `input_path` - Path to the input RAW file
/// * `output_path` - Path where the output JPEG will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted EXIF data on success
/// * `Err(String)` with error message on failure
///
/// # Example
/// ```
/// use raw_converter::convert_raw_to_jpeg;
///
/// match convert_raw_to_jpeg("photo.cr2", "photo.jpg") {
///     Ok(exif) => println!("Camera: {} {}", exif.camera_make, exif.camera_model),
///     Err(e) => eprintln!("Conversion failed: {}", e),
/// }
/// ```
pub fn convert_raw_to_jpeg(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    let input_cstring =
        CString::new(input_path).map_err(|e| format!("Invalid input path: {}", e))?;
    let output_cstring =
        CString::new(output_path).map_err(|e| format!("Invalid output path: {}", e))?;

    // Initialize empty EXIF data structure (currently unused but required by API)
    let mut exif_data = ExifData {
        camera_make: [0; 64],
        camera_model: [0; 64],
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
        // Extract EXIF data from the C structure
        let exif_info = ExifInfo {
            camera_make: safe_string_from_array(&exif_data.camera_make),
            camera_model: safe_string_from_array(&exif_data.camera_model),
            software: safe_string_from_ptr(exif_data.software),
            iso_speed: exif_data.iso_speed,
            shutter: exif_data.shutter,
            aperture: exif_data.aperture,
            focal_length: exif_data.focal_length,
            raw_width: exif_data.raw_width,
            raw_height: exif_data.raw_height,
            output_width: exif_data.output_width,
            output_height: exif_data.output_height,
            colors: exif_data.colors,
            color_filter: exif_data.color_filter,
            cam_mul: exif_data.cam_mul,
            date_taken: safe_string_from_ptr(exif_data.date_taken),
            lens: safe_string_from_ptr(exif_data.lens),
            max_aperture: exif_data.max_aperture,
            focal_length_35mm: exif_data.focal_length_35mm,
            description: safe_string_from_ptr(exif_data.description),
            artist: safe_string_from_ptr(exif_data.artist),
        };
        Ok(exif_info)
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
        || lower_name.ends_with(".cr2")   // Canon RAW (older)
        || lower_name.ends_with(".cr3")   // Canon RAW (newer)
        || lower_name.ends_with(".nef")   // Nikon RAW
        || lower_name.ends_with(".dng")   // Adobe Digital Negative
        || lower_name.ends_with(".arw")   // Sony RAW
        || lower_name.ends_with(".raf")   // Fujifilm RAW
        || lower_name.ends_with(".rw2")   // Panasonic RAW
        || lower_name.ends_with(".orf")   // Olympus RAW
        || lower_name.ends_with(".pef")   // Pentax RAW
        || lower_name.ends_with(".sr2")   // Sony RAW (older)
        || lower_name.ends_with(".srf")   // Sony RAW (older)
        || lower_name.ends_with(".srw")   // Samsung RAW
        || lower_name.ends_with(".3fr")   // Hasselblad RAW
        || lower_name.ends_with(".fff")   // Hasselblad RAW (older)
        || lower_name.ends_with(".mef")   // Mamiya RAW
        || lower_name.ends_with(".mrw")   // Minolta RAW
        || lower_name.ends_with(".x3f")   // Sigma RAW
        || lower_name.ends_with(".dcr")   // Kodak RAW
        || lower_name.ends_with(".kdc")   // Kodak RAW
        || lower_name.ends_with(".iiq")   // PhaseOne RAW
        || lower_name.ends_with(".rwl")   // Leica RAW
        || lower_name.ends_with(".gpr")   // GoPro RAW
        || lower_name.ends_with(".cap")   // PhaseOne RAW (older)
        || lower_name.ends_with(".erf")   // Epson RAW
        || lower_name.ends_with(".mdc")   // Minolta RAW (older)
        || lower_name.ends_with(".mos")   // Leaf RAW
        || lower_name.ends_with(".ptx")   // Pentax RAW (older)
        || lower_name.ends_with(".r3d") // RED RAW
}

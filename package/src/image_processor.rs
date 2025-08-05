/// Regular image processing for JPEG, PNG, TIFF, etc.
///
/// This module handles processing of standard image formats,
/// including EXIF extraction from all image files through the libjpeg wrapper.
use crate::exif_data::{ExifData, ExifInfo};
use crate::process_image_to_jpeg;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

/// Helper function to safely convert C char arrays to Rust strings (same as raw_processor)
fn safe_string_from_array(arr: &[c_char]) -> String {
    // Find the null terminator
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());

    // Convert to u8 slice and then to string
    let bytes: Vec<u8> = arr[..len].iter().map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Helper function to safely convert C string pointers to Rust strings (same as raw_processor)
fn safe_string_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}

/// Processes any image file (JPEG, PNG, TIFF, BMP, WebP, etc.) with EXIF extraction
///
/// This function handles all image formats by processing them through
/// the libjpeg wrapper to generate a JPEG preview and extract metadata.
///
/// # Arguments
/// * `input_path` - Path to the input image file
/// * `output_path` - Path where the output will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted EXIF information on success
/// * `Err(String)` with error message on failure
pub fn process_image_file(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input image file does not exist: {}", input_path));
    }

    // Convert paths to CString
    let c_input_path = CString::new(input_path).map_err(|_| "Invalid input path")?;
    let c_output_path = CString::new(output_path).map_err(|_| "Invalid output path")?;

    // Initialize EXIF data structure for libjpeg wrapper to populate (same as raw_processor)
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

    // Process image using the libjpeg wrapper
    let result = unsafe {
        process_image_to_jpeg(
            c_input_path.as_ptr(),
            c_output_path.as_ptr(),
            &mut exif_data,
        )
    };

    if result != 0 {
        return Err("Failed to process image file".to_string());
    }

    // Successfully processed - extract EXIF data from the C structure (same as raw_processor)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_nonexistent_file() {
        let result = process_image_file("nonexistent.jpg", "output.jpg");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_safe_string_from_array() {
        let test_array: [c_char; 10] = [72, 101, 108, 108, 111, 0, 0, 0, 0, 0]; // "Hello"
        let result = safe_string_from_array(&test_array);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_safe_string_from_ptr_null() {
        let result = safe_string_from_ptr(ptr::null());
        assert_eq!(result, "");
    }

    #[test]
    fn test_exif_info_for_image_types() {
        let jpeg_info = ExifInfo::for_jpeg_file();
        assert_eq!(jpeg_info.camera_model, "JPEG File");
        assert_eq!(jpeg_info.colors, 3);

        let png_info = ExifInfo::for_image_file("png");
        assert_eq!(png_info.camera_model, "PNG File");
        assert_eq!(png_info.colors, 3);
    }
}

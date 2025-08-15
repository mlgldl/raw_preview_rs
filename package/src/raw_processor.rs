/// RAW image processing using LibRaw
///
/// This module handles the conversion of RAW image files to JPEG format
/// using the LibRaw library through a C++ wrapper, with comprehensive
/// EXIF data extraction.
use crate::exif_data::{ExifData, ExifInfo};
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
    #[link_name = "process_raw_bytes_to_jpeg"]
    fn process_raw_bytes_to_jpeg_c(
        data: *const u8,
        size: usize,
        output_path: *const c_char,
        exif_data: *mut ExifData,
    ) -> i32;
    fn get_last_error() -> *const c_char;
    // free_buffer is provided by the jpeg wrapper and frees buffers allocated by the native side
    fn free_buffer(buffer: *mut u8);

    #[link_name = "process_raw_bytes_to_jpeg_buffer"]
    fn process_raw_bytes_to_jpeg_buffer_c(
        data: *const u8,
        size: usize,
        out_buf: *mut *mut u8,
        out_size: *mut usize,
        exif_data: *mut ExifData,
    ) -> i32;
}

/// Success code returned by the LibRaw wrapper
const RW_SUCCESS: i32 = 0;

/// Helper function to safely convert C char arrays to Rust strings
fn safe_string_from_array(arr: &[c_char]) -> String {
    // Find the null terminator
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());

    // Convert to u8 slice and then to string
    let bytes: Vec<u8> = arr[..len].iter().map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Helper function to safely convert C string pointers to Rust strings
fn safe_string_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}

/// Converts a RAW image file to JPEG format and extracts comprehensive EXIF data
///
/// This function uses LibRaw to process RAW files from various camera manufacturers,
/// extracting detailed metadata including camera settings, dimensions, and technical
/// parameters while converting to a high-quality JPEG output.
///
/// # Arguments
/// * `input_path` - Path to the input RAW file
/// * `output_path` - Path where the output JPEG will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted EXIF data on success
/// * `Err(String)` with detailed error message on failure
///
/// # Example
/// ```no_run
/// use raw_preview_rs::raw_processor::convert_raw_to_jpeg;
///
/// match convert_raw_to_jpeg("photo.cr2", "photo.jpg") {
///     Ok(exif) => {
///         println!("Camera: {} {}", exif.camera_make, exif.camera_model);
///         println!("ISO: {}, Aperture: {}", exif.iso_speed, exif.formatted_aperture());
///     }
///     Err(e) => eprintln!("Conversion failed: {}", e),
/// }
/// ```
///
/// # Supported RAW Formats
/// - Canon: CR2, CR3
/// - Nikon: NEF
/// - Sony: ARW, SR2, SRF
/// - Fujifilm: RAF
/// - Panasonic: RW2
/// - Olympus: ORF
/// - Pentax: PEF
/// - And many more (see file_detector module for complete list)
pub fn convert_raw_to_jpeg(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Validate and convert input paths to C strings
    let input_cstring = CString::new(input_path)
        .map_err(|e| format!("Invalid input path '{}': {}", input_path, e))?;
    let output_cstring = CString::new(output_path)
        .map_err(|e| format!("Invalid output path '{}': {}", output_path, e))?;

    // Initialize EXIF data structure for LibRaw to populate
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

    // Call the C++ LibRaw wrapper function
    let result = unsafe {
        process_raw_to_jpeg(
            input_cstring.as_ptr(),
            output_cstring.as_ptr(),
            &mut exif_data,
        )
    };

    if result == RW_SUCCESS {
        // Successfully processed - extract EXIF data from the C structure
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
        // Processing failed - retrieve detailed error message from C++ wrapper
        let error_msg = unsafe {
            let error_ptr = get_last_error();
            if !error_ptr.is_null() {
                CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
            } else {
                "Unknown LibRaw error".to_string()
            }
        };
        Err(format!("LibRaw Error {}: {}", result, error_msg))
    }
}
/// Accept RAW data as bytes and convert it to JPEG in-memory via the native FFI.
/// The resulting JPEG preview is written to the provided `output_path`.
pub fn convert_raw_bytes_to_jpeg(bytes: &[u8], output_path: &str) -> Result<ExifInfo, String> {
    let c_output = CString::new(output_path).map_err(|_| "Invalid output path")?;

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

    let ret = unsafe {
        process_raw_bytes_to_jpeg_c(
            bytes.as_ptr(),
            bytes.len(),
            c_output.as_ptr(),
            &mut exif_data,
        )
    };
    if ret == RW_SUCCESS {
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
        let error_msg = unsafe {
            let ptr = get_last_error();
            if ptr.is_null() {
                "Unknown LibRaw error".to_string()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        };
        Err(format!("LibRaw Error {}: {}", ret, error_msg))
    }
}

/// Convert RAW bytes to JPEG in-memory and return JPEG bytes + ExifInfo
pub fn convert_raw_bytes_to_vec(bytes: &[u8]) -> Result<(Vec<u8>, ExifInfo), String> {
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

    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_size: usize = 0;

    let ret = unsafe {
        process_raw_bytes_to_jpeg_buffer_c(
            bytes.as_ptr(),
            bytes.len(),
            &mut out_ptr as *mut *mut u8,
            &mut out_size as *mut usize,
            &mut exif_data,
        )
    };

    if ret != RW_SUCCESS {
        let err = unsafe {
            let p = get_last_error();
            if p.is_null() {
                "LibRaw unknown error".to_string()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        return Err(format!("LibRaw error {}: {}", ret, err));
    }

    if out_ptr.is_null() || out_size == 0 {
        return Err("No JPEG data returned".to_string());
    }

    let slice = unsafe { std::slice::from_raw_parts(out_ptr, out_size) };
    let jpeg_vec = slice.to_vec();

    // Free C-allocated buffer (allocated with new unsigned char[]) using provided free_buffer
    unsafe { free_buffer(out_ptr) };

    // Convert ExifData
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

    Ok((jpeg_vec, exif_info))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

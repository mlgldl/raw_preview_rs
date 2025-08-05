/// Regular image processing for JPEG, PNG, TIFF, etc.
///
/// This module handles processing of standard image formats,
/// including EXIF extraction from JPEG files and basic file operations
/// for other image formats.
use crate::exif_data::{ExifData, ExifInfo};
use crate::process_image_to_jpeg;
use std::ffi::CString;
use std::mem;
use std::path::Path;

/// Processes a JPEG image file with EXIF extraction
///
/// This function handles JPEG files by extracting metadata and optionally
/// copying the file to the output location.
///
/// # Arguments
/// * `input_path` - Path to the input JPEG file
/// * `output_path` - Path where the output will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted EXIF information on success
/// * `Err(String)` with error message on failure
pub fn process_jpeg_file(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input JPEG file does not exist: {}", input_path));
    }

    // Convert paths to CString
    let c_input_path = CString::new(input_path).map_err(|_| "Invalid input path")?;
    let c_output_path = CString::new(output_path).map_err(|_| "Invalid output path")?;

    // Create ExifData structure to receive metadata
    let mut exif_data: ExifData = unsafe { mem::zeroed() };

    // Process JPEG using the libjpeg wrapper
    let result = unsafe {
        process_image_to_jpeg(
            c_input_path.as_ptr(),
            c_output_path.as_ptr(),
            &mut exif_data,
        )
    };

    if result != 0 {
        return Err("Failed to process JPEG file".to_string());
    }

    // Convert C ExifData to Rust ExifInfo
    let camera_make = unsafe {
        std::ffi::CStr::from_ptr(exif_data.camera_make.as_ptr())
            .to_string_lossy()
            .to_string()
    };
    let camera_model = unsafe {
        std::ffi::CStr::from_ptr(exif_data.camera_model.as_ptr())
            .to_string_lossy()
            .to_string()
    };

    Ok(ExifInfo {
        camera_make,
        camera_model,
        iso_speed: exif_data.iso_speed,
        shutter: exif_data.shutter,
        aperture: exif_data.aperture,
        focal_length: exif_data.focal_length,
        raw_width: exif_data.raw_width,
        raw_height: exif_data.raw_height,
        output_width: exif_data.output_width,
        output_height: exif_data.output_height,
        colors: exif_data.colors,
        ..Default::default()
    })
}

/// Processes a non-JPEG image file (PNG, TIFF, BMP, WebP, etc.)
///
/// This function handles various image formats by processing them through
/// the libjpeg wrapper to generate a JPEG preview.
///
/// # Arguments
/// * `input_path` - Path to the input image file
/// * `output_path` - Path where the output will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with basic file information on success
/// * `Err(String)` with error message on failure
pub fn process_image_file(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input image file does not exist: {}", input_path));
    }

    // Convert paths to CString
    let c_input_path = CString::new(input_path).map_err(|_| "Invalid input path")?;
    let c_output_path = CString::new(output_path).map_err(|_| "Invalid output path")?;

    // Create ExifData structure to receive metadata
    let mut exif_data: ExifData = unsafe { mem::zeroed() };

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

    // Convert C ExifData to Rust ExifInfo
    let camera_make = unsafe {
        std::ffi::CStr::from_ptr(exif_data.camera_make.as_ptr())
            .to_string_lossy()
            .to_string()
    };
    let camera_model = unsafe {
        std::ffi::CStr::from_ptr(exif_data.camera_model.as_ptr())
            .to_string_lossy()
            .to_string()
    };

    Ok(ExifInfo {
        camera_make,
        camera_model,
        iso_speed: exif_data.iso_speed,
        shutter: exif_data.shutter,
        aperture: exif_data.aperture,
        focal_length: exif_data.focal_length,
        raw_width: exif_data.raw_width,
        raw_height: exif_data.raw_height,
        output_width: exif_data.output_width,
        output_height: exif_data.output_height,
        colors: exif_data.colors,
        ..Default::default()
    })
}

/// Processes any supported image file (JPEG, PNG, TIFF, etc.) with appropriate handling
///
/// This is a convenience function that automatically detects the image type
/// and applies the appropriate processing method.
///
/// # Arguments
/// * `input_path` - Path to the input image file
/// * `output_path` - Path where the output will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted/generated metadata on success
/// * `Err(String)` with error message on failure
pub fn process_any_standard_image(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    let filename = Path::new(input_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    let lower_filename = filename.to_lowercase();

    if lower_filename.ends_with(".jpg") || lower_filename.ends_with(".jpeg") {
        process_jpeg_file(input_path, output_path)
    } else {
        process_image_file(input_path, output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_nonexistent_file() {
        let result = process_jpeg_file("nonexistent.jpg", "output.jpg");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
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

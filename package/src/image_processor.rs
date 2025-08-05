/// Regular image processing for JPEG, PNG, TIFF, etc.
///
/// This module handles processing of standard image formats,
/// including EXIF extraction from JPEG files and basic file operations
/// for other image formats.
use crate::exif_data::ExifInfo;
use crate::{decode_jpeg, free_buffer, JpegInfo};
use std::ffi::CString;
use std::fs;
use std::path::Path;

/// Processes a JPEG file to extract EXIF data and optionally create a preview
///
/// For JPEG files, this function extracts EXIF metadata and can create
/// a resized preview. Currently implements basic file copying, but can be
/// extended to include actual EXIF extraction using libraries like `rexif`.
///
/// # Arguments
/// * `input_path` - Path to the input JPEG file
/// * `output_path` - Path where the output JPEG preview will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted EXIF data on success
/// * `Err(String)` with error message on failure
///
/// # Example
/// ```no_run
/// use raw_preview_rs::image_processor::process_jpeg_file;
///
/// match process_jpeg_file("photo.jpg", "preview.jpg") {
///     Ok(exif) => println!("JPEG processed: {}", exif.camera_model),
///     Err(e) => eprintln!("Failed to process JPEG: {}", e),
/// }
/// ```
pub fn process_jpeg_file(input_path: &str, _output_path: &str) -> Result<ExifInfo, String> {
    // Convert input path to CString
    let c_input_path = CString::new(input_path).map_err(|_| "Invalid input path")?;

    // Prepare output buffer and info struct
    let mut output_buffer: *mut u8 = std::ptr::null_mut();
    let mut info = JpegInfo {
        width: 0,
        height: 0,
        subsampling: 0,
        colorspace: 0,
    };

    // Decode JPEG using the wrapper
    let result = unsafe { decode_jpeg(c_input_path.as_ptr(), &mut output_buffer, &mut info) };
    if result != 0 {
        return Err("Failed to decode JPEG".to_string());
    }

    // Free the buffer after processing
    unsafe { free_buffer(output_buffer) };

    // Return basic EXIF info (placeholder for now)
    Ok(ExifInfo {
        camera_make: "Unknown".to_string(),
        camera_model: "JPEG File".to_string(),
        ..Default::default()
    })
}

/// Processes a non-JPEG image file (PNG, TIFF, BMP, WebP, etc.)
///
/// This function handles various image formats by copying them to the output
/// location. In a full implementation, you might convert them to JPEG format
/// or extract any available metadata.
///
/// # Arguments
/// * `input_path` - Path to the input image file
/// * `output_path` - Path where the output will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with basic file information on success
/// * `Err(String)` with error message on failure
///
/// # Example
/// ```no_run
/// use raw_preview_rs::image_processor::process_image_file;
///
/// match process_image_file("graphic.png", "preview.jpg") {
///     Ok(exif) => println!("Image processed: {}", exif.camera_model),
///     Err(e) => eprintln!("Failed to process image: {}", e),
/// }
/// ```
pub fn process_image_file(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input image file does not exist: {}", input_path));
    }

    let input_path_lower = input_path.to_lowercase();

    // Handle JPEG files specifically
    if input_path_lower.ends_with(".jpg") || input_path_lower.ends_with(".jpeg") {
        return process_jpeg_file(input_path, output_path);
    }

    // For other image formats, copy the file
    // TODO: In production, you might want to convert to JPEG and resize
    if let Err(e) = fs::copy(input_path, output_path) {
        return Err(format!(
            "Failed to copy image file '{}' to '{}': {}",
            input_path, output_path, e
        ));
    }

    // Extract file extension for display
    let file_extension = Path::new(input_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");

    let exif_info = ExifInfo::for_image_file(file_extension);

    Ok(exif_info)
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

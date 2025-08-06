pub mod exif_data;
/// Universal Image Processing Library
///
/// This library provides a unified interface for processing both RAW image files
/// and standard image formats (JPEG, PNG, TIFF, etc.) with comprehensive EXIF
/// data extraction and preview generation.
///
/// # Features
/// - RAW file processing using LibRaw with full EXIF extraction
/// - Standard image format handling with metadata preservation
/// - Comprehensive file type detection for 27+ RAW formats
/// - Unified API for seamless processing of any supported image format
///
/// # Example Usage
/// ```no_run
/// use raw_preview_rs::process_any_image;
///
/// match process_any_image("photo.cr2", "preview.jpg") {
///     Ok(exif) => {
///         println!("Processed: {} {}", exif.camera_make, exif.camera_model);
///         println!("Settings: ISO {}, {}, {}",
///                  exif.iso_speed, exif.formatted_aperture(), exif.formatted_shutter_speed());
///     }
///     Err(e) => eprintln!("Processing failed: {}", e),
/// }
/// ```
pub mod file_detector;
pub mod image_processor;
pub mod raw_processor;

// Re-export the main public API
pub use exif_data::ExifInfo;
pub use file_detector::{get_file_type, is_image_file, is_raw_file, is_supported_file};
pub use image_processor::process_image_file;
pub use raw_processor::convert_raw_to_jpeg;

use std::path::Path;

/// Unified function to process any supported image file (RAW or standard format)
///
/// This is the main entry point for the library, providing seamless processing
/// of any supported image format. It automatically detects the file type and
/// applies the appropriate processing method.
///
/// # Arguments
/// * `input_path` - Path to the input image file (RAW or standard format)
/// * `output_path` - Path where the output JPEG will be saved
///
/// # Returns
/// * `Ok(ExifInfo)` with extracted metadata and processing results on success
/// * `Err(String)` with detailed error message on failure
///
/// # Supported Formats
///
/// ## RAW Formats (processed via LibRaw):
/// - Canon: CR2, CR3
/// - Nikon: NEF
/// - Sony: ARW, SR2, SRF
/// - Fujifilm: RAF
/// - Panasonic: RW2
/// - Olympus: ORF
/// - Pentax: PEF, PTX
/// - Samsung: SRW
/// - Hasselblad: 3FR, FFF
/// - Mamiya: MEF
/// - Minolta: MRW, MDC
/// - Sigma: X3F
/// - Kodak: DCR, KDC
/// - PhaseOne: IIQ, CAP
/// - Leica: RWL
/// - GoPro: GPR
/// - Epson: ERF
/// - Leaf: MOS
/// - RED: R3D
/// - Adobe: DNG
/// - Generic: RAW
///
/// ## Standard Image Formats:
/// - JPEG: JPG, JPEG
/// - PNG: PNG
/// - TIFF: TIFF, TIF
/// - Bitmap: BMP
/// - WebP: WEBP
///
/// # Example
/// ```no_run
/// use raw_preview_rs::process_any_image;
///
/// // Process a Canon RAW file
/// match process_any_image("IMG_1234.CR3", "preview.jpg") {
///     Ok(exif) => println!("RAW processed: {} {}", exif.camera_make, exif.camera_model),
///     Err(e) => eprintln!("RAW processing failed: {}", e),
/// }
///
/// // Process a JPEG file
/// match process_any_image("photo.jpg", "copy.jpg") {
///     Ok(exif) => println!("JPEG processed: {}", exif.camera_model),
///     Err(e) => eprintln!("JPEG processing failed: {}", e),
/// }
/// ```
pub fn process_any_image(input_path: &str, output_path: &str) -> Result<ExifInfo, String> {
    // Extract filename for type detection
    let filename = Path::new(input_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid input path: {}", input_path))?;

    // Route to appropriate processor based on file type
    if is_raw_file(filename) {
        convert_raw_to_jpeg(input_path, output_path)
    } else if is_image_file(filename) {
        // Use image_processor for all standard image files (JPEG, PNG, TIFF, etc.)
        process_image_file(input_path, output_path)
    } else {
        Err(format!(
            "Unsupported file format: '{}'. Supported formats include RAW files (CR2, CR3, NEF, ARW, etc.) and image files (JPG, PNG, TIFF, etc.)",
            filename
        ))
    }
}

/// Checks if a file can be processed by this library
///
/// # Arguments
/// * `input_path` - Path to the file to check
///
/// # Returns
/// * `true` if the file can be processed
/// * `false` if the file format is not supported
///
/// # Example
/// ```
/// use raw_preview_rs::{can_process_file};
///
/// assert!(can_process_file("photo.cr2"));
/// assert!(can_process_file("image.jpg"));
/// // assert!(!can_process_file("document.txt"));
/// ```
pub fn can_process_file(input_path: &str) -> bool {
    let filename = Path::new(input_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    is_supported_file(filename)
}

/// Gets detailed information about file type and processing capabilities
///
/// # Arguments
/// * `input_path` - Path to the file to analyze
///
/// # Returns
/// * Information about the file type and how it would be processed
///
/// # Example
/// ```
/// use raw_preview_rs::get_file_info;
///
/// println!("{}", get_file_info("photo.cr2"));
/// // Output: "RAW file (will be processed with LibRaw)"
/// ```
pub fn get_file_info(input_path: &str) -> String {
    let filename = Path::new(input_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    match get_file_type(filename) {
        "RAW" => format!("RAW file (will be processed with LibRaw)"),
        "Image" => format!("Standard image file (will be processed with libjpeg_wrapper)"),
        _ => format!("Unsupported file format"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_process_file() {
        assert!(can_process_file("test.cr2"));
        assert!(can_process_file("image.jpg"));
        assert!(can_process_file("graphic.png"));
        assert!(!can_process_file("document.txt"));
        assert!(!can_process_file("video.mp4"));
    }

    #[test]
    fn test_get_file_info() {
        assert!(get_file_info("test.cr2").contains("RAW"));
        assert!(get_file_info("image.jpg").contains("Standard image"));
        assert!(get_file_info("document.txt").contains("Unsupported"));
    }
}

/// EXIF data structures and utilities
/// 
/// This module defines the data structures used to represent EXIF metadata
/// extracted from image files, both RAW and regular formats.

use std::os::raw::c_char;

/// C-compatible EXIF data structure for interfacing with LibRaw
/// This structure must match the ExifData struct in libraw_wrapper.h
#[repr(C)]
pub struct ExifData {
    pub camera_make: [c_char; 64],
    pub camera_model: [c_char; 64],
    pub software: *const c_char,
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
    pub date_taken: *const c_char,
    pub lens: *const c_char,
    pub max_aperture: f64,
    pub focal_length_35mm: i32,
    pub description: *const c_char,
    pub artist: *const c_char,
}

/// Rust-native EXIF data structure for safe handling
/// 
/// Represents EXIF data extracted from image files in a safe,
/// owned format that can be easily used throughout the application.
#[derive(Debug, Clone)]
pub struct ExifInfo {
    /// Camera manufacturer (e.g., "Canon", "Nikon", "Sony")
    pub camera_make: String,
    /// Camera model (e.g., "EOS R5", "D850", "A7R IV")
    pub camera_model: String,
    /// Software used to process the image
    pub software: String,
    /// ISO sensitivity setting
    pub iso_speed: i32,
    /// Shutter speed in seconds (e.g., 0.001 for 1/1000s)
    pub shutter: f64,
    /// Aperture value (e.g., 2.8 for f/2.8)
    pub aperture: f64,
    /// Focal length in millimeters
    pub focal_length: f64,
    /// Original RAW image width in pixels
    pub raw_width: i32,
    /// Original RAW image height in pixels
    pub raw_height: i32,
    /// Processed output image width in pixels
    pub output_width: i32,
    /// Processed output image height in pixels
    pub output_height: i32,
    /// Number of color channels (typically 3 for RGB, 4 for CMYK)
    pub colors: i32,
    /// Color filter array pattern (for RAW files)
    pub color_filter: i32,
    /// Camera color multipliers for white balance
    pub cam_mul: [f64; 4],
    /// Date and time when the photo was taken
    pub date_taken: String,
    /// Lens information (e.g., "24-70mm f/2.8")
    pub lens: String,
    /// Maximum aperture of the lens
    pub max_aperture: f64,
    /// Focal length equivalent in 35mm format
    pub focal_length_35mm: i32,
    /// Image description or comment
    pub description: String,
    /// Artist or photographer name
    pub artist: String,
}

impl Default for ExifInfo {
    /// Creates a default ExifInfo with empty/zero values
    fn default() -> Self {
        Self {
            camera_make: String::new(),
            camera_model: String::new(),
            software: String::new(),
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
            date_taken: String::new(),
            lens: String::new(),
            max_aperture: 0.0,
            focal_length_35mm: 0,
            description: String::new(),
            artist: String::new(),
        }
    }
}

impl ExifInfo {
    /// Creates a new ExifInfo for a JPEG file with basic information
    pub fn for_jpeg_file() -> Self {
        Self {
            camera_make: "Unknown".to_string(),
            camera_model: "JPEG File".to_string(),
            colors: 3, // Assume RGB
            cam_mul: [1.0; 4],
            ..Default::default()
        }
    }

    /// Creates a new ExifInfo for a generic image file
    pub fn for_image_file(file_extension: &str) -> Self {
        let file_type = file_extension.to_uppercase();
        Self {
            camera_make: "Unknown".to_string(),
            camera_model: format!("{} File", file_type),
            colors: 3, // Assume RGB
            cam_mul: [1.0; 4],
            ..Default::default()
        }
    }

    /// Checks if this EXIF info contains meaningful camera data
    pub fn has_camera_info(&self) -> bool {
        !self.camera_make.is_empty() 
            && !self.camera_model.is_empty()
            && self.camera_make != "Unknown"
            && !self.camera_model.contains("File")
    }

    /// Checks if this EXIF info contains exposure data
    pub fn has_exposure_info(&self) -> bool {
        self.iso_speed > 0 || self.aperture > 0.0 || self.shutter > 0.0
    }

    /// Gets a formatted shutter speed string (e.g., "1/1000s")
    pub fn formatted_shutter_speed(&self) -> String {
        if self.shutter > 0.0 {
            if self.shutter >= 1.0 {
                format!("{:.1}s", self.shutter)
            } else {
                format!("1/{:.0}s", 1.0 / self.shutter)
            }
        } else {
            "Unknown".to_string()
        }
    }

    /// Gets a formatted aperture string (e.g., "f/2.8")
    pub fn formatted_aperture(&self) -> String {
        if self.aperture > 0.0 {
            format!("f/{:.1}", self.aperture)
        } else {
            "Unknown".to_string()
        }
    }

    /// Gets image dimensions as a formatted string
    pub fn formatted_dimensions(&self) -> String {
        if self.raw_width > 0 && self.raw_height > 0 {
            format!(
                "{}x{} (RAW: {}x{})",
                self.output_width, self.output_height, self.raw_width, self.raw_height
            )
        } else if self.output_width > 0 && self.output_height > 0 {
            format!("{}x{}", self.output_width, self.output_height)
        } else {
            "Unknown".to_string()
        }
    }
}

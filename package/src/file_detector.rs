/// File type detection utilities for image processing
/// 
/// This module provides functions to identify supported file formats
/// including RAW files from various camera manufacturers and standard image formats.

/// Checks if a file extension corresponds to a supported RAW format
///
/// # Arguments
/// * `filename` - The filename to check
///
/// # Returns
/// * `true` if the file extension is a supported RAW format
/// * `false` otherwise
/// 
/// # Supported RAW Formats
/// - Canon: .cr2, .cr3
/// - Nikon: .nef
/// - Sony: .arw, .sr2, .srf
/// - Fujifilm: .raf
/// - Panasonic: .rw2
/// - Olympus: .orf
/// - Pentax: .pef, .ptx
/// - Samsung: .srw
/// - Hasselblad: .3fr, .fff
/// - And many more professional formats
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
        || lower_name.ends_with(".r3d")   // RED RAW
}

/// Checks if a file extension corresponds to a supported image format (JPEG, PNG, etc.)
///
/// # Arguments
/// * `filename` - The filename to check
///
/// # Returns
/// * `true` if the file extension is a supported image format
/// * `false` otherwise
/// 
/// # Supported Image Formats
/// - JPEG: .jpg, .jpeg
/// - PNG: .png
/// - TIFF: .tiff, .tif
/// - Bitmap: .bmp
/// - WebP: .webp
pub fn is_image_file(filename: &str) -> bool {
    let lower_name = filename.to_lowercase();
    lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")  // JPEG images
        || lower_name.ends_with(".png")   // PNG images
        || lower_name.ends_with(".tiff")  // TIFF images
        || lower_name.ends_with(".tif")   // TIFF images (short)
        || lower_name.ends_with(".bmp")   // Bitmap images
        || lower_name.ends_with(".webp")  // WebP images
}

/// Checks if a file is supported by this image processor
///
/// # Arguments
/// * `filename` - The filename to check
///
/// # Returns
/// * `true` if the file is either a RAW file or a supported image format
/// * `false` otherwise
pub fn is_supported_file(filename: &str) -> bool {
    is_raw_file(filename) || is_image_file(filename)
}

/// Gets the file type category as a string for display purposes
///
/// # Arguments
/// * `filename` - The filename to check
///
/// # Returns
/// * "RAW" if it's a RAW file
/// * "Image" if it's a regular image file
/// * "Unknown" if it's not a supported format
pub fn get_file_type(filename: &str) -> &'static str {
    if is_raw_file(filename) {
        "RAW"
    } else if is_image_file(filename) {
        "Image"
    } else {
        "Unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_file_detection() {
        assert!(is_raw_file("test.cr2"));
        assert!(is_raw_file("TEST.CR3"));
        assert!(is_raw_file("photo.nef"));
        assert!(is_raw_file("image.arw"));
        assert!(!is_raw_file("photo.jpg"));
        assert!(!is_raw_file("document.txt"));
    }

    #[test]
    fn test_image_file_detection() {
        assert!(is_image_file("photo.jpg"));
        assert!(is_image_file("IMAGE.JPEG"));
        assert!(is_image_file("graphic.png"));
        assert!(is_image_file("scan.tiff"));
        assert!(!is_image_file("photo.cr2"));
        assert!(!is_image_file("document.txt"));
    }

    #[test]
    fn test_supported_file_detection() {
        assert!(is_supported_file("photo.jpg"));
        assert!(is_supported_file("image.cr2"));
        assert!(is_supported_file("graphic.png"));
        assert!(!is_supported_file("document.txt"));
        assert!(!is_supported_file("video.mp4"));
    }

    #[test]
    fn test_file_type_detection() {
        assert_eq!(get_file_type("photo.cr2"), "RAW");
        assert_eq!(get_file_type("image.jpg"), "Image");
        assert_eq!(get_file_type("document.txt"), "Unknown");
    }
}

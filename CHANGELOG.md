# Changelog

All notable changes to this project will be documented in this file.

## [0.1.2] - 2025-08-15

### Added

-   In-memory processing APIs: accept image or RAW data as byte slices and process them without temporary files.

    -   `process_image_bytes(bytes: &[u8], output_path: &str) -> Result<ExifInfo, String>`
    -   `convert_raw_bytes_to_jpeg(bytes: &[u8], output_path: &str) -> Result<ExifInfo, String>`

-   Vec-returning APIs (return JPEG bytes in-memory):
    -   `process_image_bytes_to_vec(bytes: &[u8]) -> Result<(Vec<u8>, ExifInfo), String>`
    -   `convert_raw_bytes_to_vec(bytes: &[u8]) -> Result<(Vec<u8>, ExifInfo), String>`

### Changed

-   README updated to reflect the current public API surface.

## [0.1.1] - 2025-08-06

### Fixed

-   doc.rs build fail

## Updated

-   README.md
-   minor code refactor

## [0.1.0] - 2025-08-05

### Added

-   Initial release of `raw_preview_rs`.
-   Support for 27+ RAW formats including CR2, NEF, ARW, RAF, and more.
-   Standard image format support: JPEG, PNG, TIFF, BMP, and WebP.
-   EXIF metadata extraction including camera make, model, ISO, and more.
-   Automatic resolution reduction for fast previews.
-   Statically linked dependencies for high performance and broad format support.

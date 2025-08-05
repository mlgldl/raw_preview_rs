# raw_preview_rs

![Crates.io](https://img.shields.io/crates/v/raw_preview_rs)
![Docs.rs](https://docs.rs/raw_preview_rs/badge.svg)
![License](https://img.shields.io/crates/l/raw_preview_rs)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

A Rust library designed to quickly create preview JPEGs from RAW image files and extract comprehensive EXIF metadata. This library is optimized for speed and efficiency, making it ideal for applications that require fast image previews and the EXIF metadata.

## Features

-   **RAW Image Processing**: Supports 27+ RAW formats, including CR2, NEF, ARW, RAF, and more.
-   **Standard Image Formats**: Handles JPEG, PNG, TIFF, BMP, and WebP.
-   **EXIF Metadata Extraction**: Extracts and preserves EXIF metadata, including camera make, model, ISO, and more.
-   **Resolution Reduction**: Automatically reduces image resolution for fast previews.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
raw_preview_rs = "0.1.0"
```

Or, if using the GitHub repository:

```toml
[dependencies]
raw_preview_rs = { git = "https://github.com/mlgldl/raw_preview_rs" }
```

## Usage

### Example: Processing a RAW File

```rust
use raw_preview_rs::process_any_image;

match process_any_image("photo.cr2", "preview.jpg") {
    Ok(exif) => {
        println!("Processed: {} {}", exif.camera_make, exif.camera_model);
        println!("Settings: ISO {}, {}, {}",
                 exif.iso_speed, exif.formatted_aperture(), exif.formatted_shutter_speed());
    }
    Err(e) => eprintln!("Processing failed: {}", e),
}
```

### Example: Processing a JPEG File

```rust
use raw_preview_rs::process_any_image;

match process_any_image("photo.jpg", "copy.jpg") {
    Ok(exif) => println!("JPEG processed: {}", exif.camera_model),
    Err(e) => eprintln!("JPEG processing failed: {}", e),
}
```

## Supported Formats

### RAW Formats (processed via LibRaw):

-   Canon: CR2, CR3
-   Nikon: NEF
-   Sony: ARW, SR2, SRF
-   Fujifilm: RAF
-   Panasonic: RW2
-   Olympus: ORF
-   Pentax: PEF, PTX
-   Samsung: SRW
-   Hasselblad: 3FR, FFF
-   Mamiya: MEF
-   Minolta: MRW, MDC
-   Sigma: X3F
-   Kodak: DCR, KDC
-   PhaseOne: IIQ, CAP
-   Leica: RWL
-   GoPro: GPR
-   Epson: ERF
-   Leaf: MOS
-   RED: R3D
-   Adobe: DNG
-   Generic: RAW

### Standard Image Formats:

-   JPEG: JPG, JPEG
-   PNG: PNG
-   TIFF: TIFF, TIF
-   Bitmap: BMP
-   WebP: WEBP

## External Dependencies

This library relies on several external C and C++ dependencies to provide efficient image processing and metadata handling. Below is a list of these dependencies with links to their respective repositories:

-   **[TurboJPEG](https://github.com/libjpeg-turbo/libjpeg-turbo)**: Used for fast JPEG compression and decompression.
-   **[stb_image](https://github.com/nothings/stb)**: A lightweight library for decoding standard image formats like PNG, BMP, and JPEG.
-   **[TinyEXIF](https://github.com/cdcseacave/TinyEXIF)**: A small library for extracting EXIF metadata from JPEG files.
-   **[LibRaw](https://www.libraw.org/)**: A library for decoding RAW image formats from various camera manufacturers.

All dependencies are statically linked, meaning they are bundled directly into the library during the build process. As a result, there is no need to install these dependencies separately on your system.

These dependencies are integrated into the library to ensure high performance and broad format support. Make sure to have the necessary build tools installed to compile these dependencies when building the library.

## License

This project is licensed under the GNU General Public License (GPL) version 3. See the [LICENSE](LICENSE) file for details.

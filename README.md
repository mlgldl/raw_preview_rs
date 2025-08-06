# raw_preview_rs

[![Crates.io](https://img.shields.io/crates/v/raw_preview_rs)](https://crates.io/crates/raw_preview_rs)
[![Documentation](https://docs.rs/raw_preview_rs/badge.svg)](https://docs.rs/raw_preview_rs)
[![License](https://img.shields.io/crates/l/raw_preview_rs)](https://github.com/mlgldl/raw_preview_rs/blob/master/LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

A Rust library designed to quickly create preview JPEGs from RAW image files and extract comprehensive EXIF metadata.\
‼️ This library is optimized for compatability, hence it requires an involved build process to statically link the C/C++ dependencies.

This library/crate is in early development.

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

### First Build

The first build will take longer (5-15 minutes) as it downloads and compiles all native dependencies. Subsequent builds will be much faster as dependencies are cached.

```bash
# Clean build (if needed)
cargo clean

# Build with full output
cargo build --release

# Or build and run tests
cargo test
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

## Build Requirements

This library has several native dependencies that are automatically downloaded and built during compilation. To ensure a successful build, you need the following tools installed on your system:

### Required Build Tools

#### All Platforms

-   **CMake** (version 3.10 or higher)

    -   Used for building TinyEXIF and TinyXML2
    -   Download from: https://cmake.org/download/

-   **Make**
    -   Used for building all dependencies
    -   Usually pre-installed on Unix-like systems

#### macOS

```bash
# Install Xcode Command Line Tools (includes make, clang, etc.)
xcode-select --install

# Install CMake using Homebrew
brew install cmake

# Install autotools (recommended for LibRaw)
brew install autoconf automake libtool pkg-config
```

#### Ubuntu/Debian

```bash
# Install build essentials and CMake
sudo apt update
sudo apt install build-essential cmake

# Install autotools (recommended for LibRaw)
sudo apt install autoconf automake libtool pkg-config
```

#### CentOS/RHEL/Fedora

```bash
# Install build tools and CMake
sudo yum groupinstall "Development Tools"
sudo yum install cmake

# Install autotools (recommended for LibRaw)
sudo yum install autoconf automake libtool pkgconfig
```

#### Windows

-   **Visual Studio** (2019 or later) with C++ build tools
-   **CMake** - Download from https://cmake.org/download/
-   **Git** (for downloading dependencies)

Alternatively, use **vcpkg** or **Conan** to manage dependencies.

### Optional Tools

-   **autoconf**, **automake**, **libtool**: Recommended for building LibRaw from source
    -   If these are not available, the build script will attempt to use pre-generated configure scripts
    -   Without these tools, some LibRaw versions may fail to build

### Automatic Dependency Management

The following dependencies are automatically downloaded and built during compilation:

1. **zlib 1.3** - Compression library
2. **LibRaw 0.21.4** - RAW image processing
3. **libjpeg-turbo 2.1.5** - JPEG compression/decompression
4. **TinyEXIF 1.0.3** - EXIF metadata extraction
5. **TinyXML2 11.0.0** - XML parsing for XMP metadata
6. **stb_image** - Standard image format decoding

All dependencies are statically linked into the final library, so end users don't need to install anything separately.

### Build Troubleshooting

If you encounter build issues:

1. **Missing autotools**: Install autoconf, automake, and libtool
2. **CMake not found**: Ensure CMake is in your PATH
3. **Compiler errors**: Ensure you have a C++11 compatible compiler
4. **Network issues**: The build downloads dependencies from the internet

For detailed error messages, run:

```bash
RUST_BACKTRACE=1 cargo build
```

### Cross-compilation

Cross-compilation is supported but requires the target platform's build tools. Ensure CMake and make are available for your target platform.

## Dependencies

This library automatically manages all its native dependencies through a custom build script. The following libraries are downloaded, compiled, and statically linked during the build process:

| Dependency        | Version | Purpose                        | Source                                                   |
| ----------------- | ------- | ------------------------------ | -------------------------------------------------------- |
| **zlib**          | 1.3     | Compression library            | [zlib.net](https://zlib.net/)                            |
| **LibRaw**        | 0.21.4  | RAW image processing           | [GitHub](https://github.com/LibRaw/LibRaw)               |
| **libjpeg-turbo** | 2.1.5   | JPEG compression/decompression | [GitHub](https://github.com/libjpeg-turbo/libjpeg-turbo) |
| **TinyEXIF**      | 1.0.3   | EXIF metadata extraction       | [GitHub](https://github.com/cdcseacave/TinyEXIF)         |
| **TinyXML2**      | 11.0.0  | XML parsing for XMP metadata   | [GitHub](https://github.com/leethomason/tinyxml2)        |
| **stb_image**     | latest  | Standard image format decoding | [GitHub](https://github.com/nothings/stb)                |

### Dependency Management Features

-   **Automatic Download**: All dependencies are downloaded from their official sources
-   **Version Pinning**: Specific versions are used to ensure build reproducibility
-   **Static Linking**: All libraries are statically linked for easy deployment
-   **Caching**: Built dependencies are cached to speed up subsequent builds
-   **Cross-platform**: Works on macOS, Linux, and Windows with appropriate build tools

No manual dependency installation is required - just ensure you have the build tools listed above.

## License

This project is licensed under the GNU General Public License (GPL) version 3. See the [LICENSE](LICENSE) file for details.

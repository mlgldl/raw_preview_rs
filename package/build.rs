use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use flate2;
use reqwest;
use tar;

// Dependency configuration (reserved for future use)
#[allow(dead_code)]
struct Dependency {
    name: &'static str,
    url: &'static str,
    extract_dir: &'static str,
    target_dir: &'static str,
}

#[allow(dead_code)]
const DEPENDENCIES: &[Dependency] = &[
    Dependency {
        name: "zlib",
        url: "https://zlib.net/fossils/zlib-1.3.tar.gz",
        extract_dir: "zlib-1.3",
        target_dir: "zlib",
    },
    Dependency {
        name: "LibRaw",
        url: "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.4.tar.gz",
        extract_dir: "LibRaw-0.21.4",
        target_dir: "LibRaw",
    },
    Dependency {
        name: "libjpeg-turbo",
        url: "https://github.com/libjpeg-turbo/libjpeg-turbo/releases/download/2.1.5/libjpeg-turbo-2.1.5.tar.gz",
        extract_dir: "libjpeg-turbo-2.1.5",
        target_dir: "libjpeg-turbo",
    },
    Dependency {
        name: "TinyEXIF",
        url: "https://github.com/cdcseacave/TinyEXIF/archive/refs/tags/1.0.3.tar.gz",
        extract_dir: "TinyEXIF-1.0.3",
        target_dir: "TinyEXIF",
    },
    Dependency {
        name: "TinyXML2",
        url: "https://github.com/leethomason/tinyxml2/archive/refs/tags/11.0.0.tar.gz",
        extract_dir: "tinyxml2-11.0.0",
        target_dir: "tinyxml2",
    },
];

struct BuildPaths {
    zlib_src: String,
    libraw_src: String,
    libjpeg_src: String,
    tinyexif_src: String,
    tinyxml2_src: String,
    tinyxml2_build: String,
    stb_dir: String,
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Check for required build tools
    check_build_tools();

    // Build all dependencies
    let paths = build_all_dependencies(&out_dir);

    // Configure linking
    configure_linking(&paths);

    // Compile C++ wrappers
    compile_wrappers(&paths);

    // Tell cargo to rerun this build script if these files change
    println!("cargo:rerun-if-changed=libraw_wrapper.cpp");
    println!("cargo:rerun-if-changed=libraw_wrapper.h");
    println!("cargo:rerun-if-changed=libjpeg_wrapper.cpp");
    println!("cargo:rerun-if-changed=libjpeg_wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
}

fn check_build_tools() {
    let required_tools = vec![
        (
            "cmake",
            "CMake is required for building TinyEXIF and TinyXML2",
        ),
        ("make", "Make is required for building all dependencies"),
    ];

    for (tool, message) in required_tools {
        if Command::new(tool).arg("--version").output().is_err() {
            panic!("{} not found. {}", tool, message);
        }
    }

    // Check for autotools (optional but recommended)
    if Command::new("autoreconf")
        .arg("--version")
        .output()
        .is_err()
    {
        println!(
            "cargo:warning=autoreconf not found. This may cause issues building LibRaw from source."
        );
        println!(
            "cargo:warning=Consider installing autotools: brew install autoconf automake libtool"
        );
    }
}

fn build_all_dependencies(out_dir: &str) -> BuildPaths {
    // --- ZLIB ---
    let zlib_dir = Path::new(out_dir).join("zlib");
    let zlib_src_dir = zlib_dir.join("zlib-1.3");
    let zlib_lib = zlib_src_dir.join("libz.a");

    if !zlib_lib.exists() {
        println!("cargo:warning=Downloading and building zlib...");
        download_and_extract_zlib(&zlib_dir, "https://zlib.net/fossils/zlib-1.3.tar.gz");
        build_zlib(&zlib_src_dir);
    }

    // --- LIBRAW ---
    let libraw_dir = Path::new(out_dir).join("LibRaw");
    let libraw_lib = libraw_dir.join("lib").join("libraw.a");
    let libraw_configure = libraw_dir.join("configure");

    if !libraw_lib.exists() || !libraw_configure.exists() {
        println!("cargo:warning=Downloading and building LibRaw...");
        download_and_extract_libraw(
            out_dir,
            "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.4.tar.gz",
        );
        build_libraw_with_zlib(&libraw_dir, &zlib_src_dir);
    }

    // --- LIBJPEG-TURBO ---
    let libjpeg_dir = Path::new(out_dir).join("libjpeg-turbo");
    let libjpeg_src_dir = libjpeg_dir.join("libjpeg-turbo-2.1.5");
    let libjpeg_lib = libjpeg_src_dir.join("build").join("libjpeg.a");

    if !libjpeg_lib.exists() {
        println!("cargo:warning=Downloading and building libjpeg-turbo...");
        download_and_extract_libjpeg(
            &libjpeg_dir,
            "https://github.com/libjpeg-turbo/libjpeg-turbo/releases/download/2.1.5/libjpeg-turbo-2.1.5.tar.gz",
        );
        build_libjpeg(&libjpeg_src_dir);
    }

    // --- TINYEXIF ---
    let tinyexif_dir = Path::new(out_dir).join("TinyEXIF");
    let tinyexif_src_dir = tinyexif_dir.join("TinyEXIF-1.0.3");

    if !tinyexif_src_dir.exists() {
        println!("cargo:warning=Downloading and setting up TinyEXIF...");
        download_and_extract_tinyexif(
            &tinyexif_dir,
            "https://github.com/cdcseacave/TinyEXIF/archive/refs/tags/1.0.3.tar.gz",
        );
    }

    // --- TINYXML2 ---
    let tinyxml2_dir = Path::new(out_dir).join("tinyxml2");
    let tinyxml2_src_dir = tinyxml2_dir.join("tinyxml2-11.0.0");

    if !tinyxml2_src_dir.exists() {
        println!("cargo:warning=Downloading and setting up TinyXML2...");
        download_and_extract_tinyxml2(
            &tinyxml2_dir,
            "https://github.com/leethomason/tinyxml2/archive/refs/tags/11.0.0.tar.gz",
        );
    }

    // Build TinyXML2
    let tinyxml2_build_dir = tinyxml2_src_dir.join("build");
    build_tinyxml2(&tinyxml2_src_dir, &tinyxml2_build_dir);

    // Build TinyEXIF
    build_tinyexif(&tinyexif_src_dir, &tinyxml2_build_dir);

    // --- STB_IMAGE ---
    let stb_dir = Path::new(out_dir).join("stb");
    let stb_image_header = stb_dir.join("stb_image.h");

    if !stb_image_header.exists() {
        println!("cargo:warning=Downloading stb_image.h...");
        download_stb_image(&stb_dir);
    }

    BuildPaths {
        zlib_src: zlib_src_dir.display().to_string(),
        libraw_src: libraw_dir.display().to_string(),
        libjpeg_src: libjpeg_src_dir.display().to_string(),
        tinyexif_src: tinyexif_src_dir.display().to_string(),
        tinyxml2_src: tinyxml2_src_dir.display().to_string(),
        tinyxml2_build: tinyxml2_build_dir.display().to_string(),
        stb_dir: stb_dir.display().to_string(),
    }
}

fn configure_linking(paths: &BuildPaths) {
    // Tell cargo to look for static libraries
    println!("cargo:rustc-link-search=native={}/lib", paths.libraw_src);
    println!("cargo:rustc-link-search=native={}", paths.zlib_src);
    println!("cargo:rustc-link-search=native={}/build", paths.libjpeg_src);
    println!("cargo:rustc-link-search=native={}", paths.tinyexif_src);
    println!("cargo:rustc-link-search=native={}", paths.tinyxml2_build);

    // Link statically against libraries
    println!("cargo:rustc-link-lib=static=raw");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=static=jpeg");
    println!("cargo:rustc-link-lib=static=turbojpeg");
    println!("cargo:rustc-link-lib=static=TinyEXIF");
    println!("cargo:rustc-link-lib=static=tinyxml2");
    println!("cargo:rustc-link-lib=m"); // math library
    println!("cargo:rustc-link-lib=c++"); // C++ standard library (macOS)
}

fn compile_wrappers(paths: &BuildPaths) {
    // Compile LibRaw wrapper
    cc::Build::new()
        .cpp(true)
        .file("libraw_wrapper.cpp")
        .include(&paths.libraw_src)
        .include(&paths.zlib_src)
        .include(&paths.libjpeg_src)
        .flag("-std=c++11")
        .flag("-O3")
        .flag("-DLIBRAW_NOTHREADS")
        .flag("-DUSE_ZLIB")
        .compile("raw_wrapper");

    // Compile libjpeg wrapper
    cc::Build::new()
        .cpp(true)
        .file("libjpeg_wrapper.cpp")
        .include(&paths.libjpeg_src)
        .include(&paths.tinyexif_src)
        .include(&paths.tinyxml2_src)
        .include(&paths.stb_dir)
        .file(format!("{}/TinyEXIF.cpp", paths.tinyexif_src))
        .flag("-std=c++11")
        .flag("-O3")
        .compile("jpeg_wrapper");
}

// Download and extraction functions
fn download_and_extract_zlib(out_dir: &Path, url: &str) {
    let zlib_extract_dir = out_dir.join("zlib-1.3");

    if zlib_extract_dir.exists() {
        fs::remove_dir_all(&zlib_extract_dir).expect("Failed to remove existing zlib directory");
    }

    fs::create_dir_all(out_dir).expect("Failed to create zlib dir");
    let resp = reqwest::blocking::get(url).expect("Failed to download zlib");
    if !resp.status().is_success() {
        panic!("Failed to download zlib: HTTP {}", resp.status());
    }
    let response = resp.bytes().expect("Failed to read zlib download").to_vec();
    let tar = flate2::read::GzDecoder::new(std::io::Cursor::new(response));
    let mut archive = tar::Archive::new(tar);
    archive.unpack(out_dir).expect("Failed to extract zlib");

    // Patch zutil.h to avoid fdopen macro redefinition on macOS
    #[cfg(target_os = "macos")]
    {
        use std::io::{Read, Write};
        let zutil_path = out_dir.join("zlib-1.3").join("zutil.h");
        if zutil_path.exists() {
            let mut contents = String::new();
            {
                let mut file = fs::File::open(&zutil_path).expect("Failed to open zutil.h");
                file.read_to_string(&mut contents)
                    .expect("Failed to read zutil.h");
            }
            let patched = contents.replace(
                "#        define fdopen(fd,mode) NULL /* No fdopen() */",
                "// #        define fdopen(fd,mode) NULL /* No fdopen() */",
            );
            let mut file = fs::File::create(&zutil_path).expect("Failed to write zutil.h");
            file.write_all(patched.as_bytes())
                .expect("Failed to patch zutil.h");
        }
    }
}

fn build_zlib(zlib_src_dir: &Path) {
    let output = Command::new("sh")
        .arg("configure")
        .current_dir(zlib_src_dir)
        .output()
        .expect("Failed to run zlib configure");
    if !output.status.success() {
        panic!(
            "Failed to configure zlib: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new("make")
        .arg("libz.a")
        .current_dir(zlib_src_dir)
        .output()
        .expect("Failed to build zlib");
    if !output.status.success() {
        panic!(
            "Failed to build zlib: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn download_and_extract_libraw(out_dir: &str, url: &str) {
    let target_dir = Path::new(out_dir).join("LibRaw");

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).expect("Failed to remove existing LibRaw directory");
    }

    fs::create_dir_all(out_dir).expect("Failed to create LibRaw dir");
    let resp = reqwest::blocking::get(url).expect("Failed to download LibRaw");
    if !resp.status().is_success() {
        panic!("Failed to download LibRaw: HTTP {}", resp.status());
    }
    let response = resp
        .bytes()
        .expect("Failed to read LibRaw download")
        .to_vec();
    let tar = flate2::read::GzDecoder::new(std::io::Cursor::new(response));
    let mut archive = tar::Archive::new(tar);
    archive.unpack(out_dir).expect("Failed to extract LibRaw");

    // Handle different extraction directory names
    let possible_dirs = vec![
        Path::new(out_dir).join("LibRaw-0.21.4"),
        Path::new(out_dir).join("LibRaw-master"),
        Path::new(out_dir).join("LibRaw"),
    ];

    for extracted_dir in possible_dirs {
        if extracted_dir.exists() && extracted_dir != target_dir {
            fs::rename(extracted_dir, &target_dir).expect("Failed to rename LibRaw directory");
            break;
        }
    }
}

fn build_libraw_with_zlib(libraw_dir: &Path, zlib_src_dir: &Path) {
    let lib_dir = libraw_dir.join("lib");
    fs::create_dir_all(&lib_dir).expect("Failed to create lib directory");

    // First run autoreconf to generate configure script if needed
    if !libraw_dir.join("configure").exists() {
        let output = Command::new("autoreconf")
            .arg("-fiv")
            .current_dir(libraw_dir)
            .output();
        match output {
            Ok(result) => {
                if !result.status.success() {
                    println!("cargo:warning=autoreconf failed, trying without it");
                }
            }
            Err(_) => {
                println!("cargo:warning=autoreconf not found, skipping");
            }
        }
    }

    // Configure LibRaw with static zlib
    let zlib_include = zlib_src_dir.to_str().unwrap();
    let zlib_lib = zlib_src_dir.to_str().unwrap();
    let mut configure = Command::new("./configure");
    configure
        .arg("--disable-shared")
        .arg("--enable-static")
        .arg("--disable-examples")
        .arg("--disable-openmp")
        .arg("--disable-lcms")
        .arg("--disable-jasper")
        .arg("--disable-jpeg")
        .arg("--disable-rawspeed")
        .arg("--disable-demosaic-pack-GPL2")
        .arg("--disable-demosaic-pack-GPL3")
        .arg("--disable-demosaic-pack-LGPL")
        .env("CPPFLAGS", format!("-I{}", zlib_include))
        .env("LDFLAGS", format!("-L{}", zlib_lib));
    configure.current_dir(libraw_dir);
    let output = configure
        .output()
        .expect("Failed to execute configure command");
    if !output.status.success() {
        panic!(
            "Failed to configure LibRaw: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Build LibRaw using make
    let output = Command::new("make")
        .arg("lib/libraw.la")
        .current_dir(libraw_dir)
        .output()
        .expect("Failed to execute make command");
    if !output.status.success() {
        panic!(
            "Failed to build LibRaw: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Copy the built library to lib directory
    let possible_sources = vec![
        libraw_dir.join("lib").join(".libs").join("libraw.a"),
        libraw_dir.join("lib").join("libraw.a"),
        libraw_dir.join(".libs").join("libraw.a"),
        libraw_dir.join("libraw.a"),
        libraw_dir.join("object").join("libraw.a"),
    ];
    let dst_lib = lib_dir.join("libraw.a");
    let mut found = false;
    for src in possible_sources {
        if src.exists() {
            fs::copy(&src, &dst_lib).expect("Failed to copy libraw.a");
            found = true;
            break;
        }
    }
    if !found {
        panic!("Could not find built libraw.a library");
    }
}

fn download_and_extract_libjpeg(out_dir: &Path, url: &str) {
    let libjpeg_extract_dir = out_dir.join("libjpeg-turbo-2.1.5");

    if libjpeg_extract_dir.exists() {
        fs::remove_dir_all(&libjpeg_extract_dir)
            .expect("Failed to remove existing libjpeg-turbo directory");
    }

    fs::create_dir_all(out_dir).expect("Failed to create libjpeg-turbo dir");
    let resp = reqwest::blocking::get(url).expect("Failed to download libjpeg-turbo");
    if !resp.status().is_success() {
        panic!("Failed to download libjpeg-turbo: HTTP {}", resp.status());
    }
    let response = resp
        .bytes()
        .expect("Failed to read libjpeg-turbo download")
        .to_vec();
    let tar = flate2::read::GzDecoder::new(std::io::Cursor::new(response));
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(out_dir)
        .expect("Failed to extract libjpeg-turbo");
}

fn build_libjpeg(libjpeg_src_dir: &Path) {
    let build_dir = libjpeg_src_dir.join("build");
    fs::create_dir_all(&build_dir).expect("Failed to create build directory for libjpeg-turbo");

    let output = Command::new("cmake")
        .arg("..")
        .arg("-DENABLE_STATIC=1")
        .arg("-DENABLE_SHARED=0")
        .arg("-DWITH_TURBOJPEG=1") // Enable TurboJPEG API
        .arg("-DCMAKE_OSX_ARCHITECTURES=arm64") // Ensure correct architecture
        .arg("-DCMAKE_OSX_DEPLOYMENT_TARGET=15.0") // Update deployment target to 15.0
        .current_dir(&build_dir)
        .output()
        .expect("Failed to configure libjpeg-turbo");
    if !output.status.success() {
        panic!(
            "Failed to configure libjpeg-turbo: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("make")
        .current_dir(&build_dir)
        .output()
        .expect("Failed to build libjpeg-turbo");
    if !output.status.success() {
        panic!(
            "Failed to build libjpeg-turbo: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Copy the built library to lib directory
    let lib_dir = libjpeg_src_dir.join("lib");
    fs::create_dir_all(&lib_dir).expect("Failed to create lib directory for libjpeg-turbo");
    let built_lib = build_dir.join("libjpeg.a");
    let dst_lib = lib_dir.join("libjpeg.a");
    fs::copy(&built_lib, &dst_lib).expect("Failed to copy libjpeg.a");
}

fn download_and_extract_tinyxml2(out_dir: &Path, url: &str) {
    let tinyxml2_extract_dir = out_dir.join("tinyxml2-11.0.0");

    if tinyxml2_extract_dir.exists() {
        fs::remove_dir_all(&tinyxml2_extract_dir)
            .expect("Failed to remove existing TinyXML2 directory");
    }

    fs::create_dir_all(out_dir).expect("Failed to create TinyXML2 dir");
    let resp = reqwest::blocking::get(url).expect("Failed to download TinyXML2");
    if !resp.status().is_success() {
        panic!("Failed to download TinyXML2: HTTP {}", resp.status());
    }
    let response = resp
        .bytes()
        .expect("Failed to read TinyXML2 download")
        .to_vec();
    let tar = flate2::read::GzDecoder::new(std::io::Cursor::new(response));
    let mut archive = tar::Archive::new(tar);
    archive.unpack(out_dir).expect("Failed to extract TinyXML2");
}

fn build_tinyxml2(_src_dir: &Path, build_dir: &Path) {
    fs::create_dir_all(build_dir).expect("Failed to create build directory for TinyXML2");

    let output = Command::new("cmake")
        .arg("..")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DBUILD_STATIC_LIBS=ON")
        .arg("-DCMAKE_INSTALL_PREFIX=.")
        .current_dir(build_dir)
        .output()
        .expect("Failed to configure TinyXML2");
    if !output.status.success() {
        panic!(
            "Failed to configure TinyXML2: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("make")
        .current_dir(build_dir)
        .output()
        .expect("Failed to build TinyXML2");
    if !output.status.success() {
        panic!(
            "Failed to build TinyXML2: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Install TinyXML2
    let output = Command::new("make")
        .arg("install")
        .current_dir(build_dir)
        .output()
        .expect("Failed to install TinyXML2");
    if !output.status.success() {
        panic!(
            "Failed to install TinyXML2: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn download_and_extract_tinyexif(out_dir: &Path, url: &str) {
    let tinyexif_extract_dir = out_dir.join("TinyEXIF-1.0.3");

    if tinyexif_extract_dir.exists() {
        fs::remove_dir_all(&tinyexif_extract_dir)
            .expect("Failed to remove existing TinyEXIF directory");
    }

    fs::create_dir_all(out_dir).expect("Failed to create TinyEXIF dir");
    let resp = reqwest::blocking::get(url).expect("Failed to download TinyEXIF");
    if !resp.status().is_success() {
        panic!("Failed to download TinyEXIF: HTTP {}", resp.status());
    }
    let response = resp
        .bytes()
        .expect("Failed to read TinyEXIF download")
        .to_vec();
    let tar = flate2::read::GzDecoder::new(std::io::Cursor::new(response));
    let mut archive = tar::Archive::new(tar);
    archive.unpack(out_dir).expect("Failed to extract TinyEXIF");
}

fn build_tinyexif(src_dir: &Path, tinyxml2_build_dir: &Path) {
    let tinyxml2_install_dir = tinyxml2_build_dir.display().to_string();

    let output = Command::new("cmake")
        .arg(".")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DBUILD_STATIC_LIBS=ON")
        .arg("-DTINYEXIF_NO_XMP=OFF") // Enable XMP parsing
        .arg(format!("-DCMAKE_PREFIX_PATH={}", tinyxml2_install_dir))
        .current_dir(src_dir)
        .output()
        .expect("Failed to configure TinyEXIF");
    if !output.status.success() {
        panic!(
            "Failed to configure TinyEXIF: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("make")
        .current_dir(src_dir)
        .output()
        .expect("Failed to build TinyEXIF");
    if !output.status.success() {
        panic!(
            "Failed to build TinyEXIF: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn download_stb_image(stb_dir: &Path) {
    fs::create_dir_all(stb_dir).expect("Failed to create stb dir");

    let stb_image_url = "https://raw.githubusercontent.com/nothings/stb/master/stb_image.h";
    let resp = reqwest::blocking::get(stb_image_url).expect("Failed to download stb_image.h");
    if !resp.status().is_success() {
        panic!("Failed to download stb_image.h: HTTP {}", resp.status());
    }

    let content = resp.text().expect("Failed to read stb_image.h content");
    let stb_image_path = stb_dir.join("stb_image.h");
    fs::write(stb_image_path, content).expect("Failed to write stb_image.h");
}

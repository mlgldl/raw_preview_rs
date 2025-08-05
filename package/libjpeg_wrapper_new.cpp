#include <turbojpeg.h>
#include <iostream>
#include <fstream>
#include <vector>
#include <cstring>
#include "TinyEXIF.h" // Include TinyEXIF header
#include "libjpeg_wrapper.h"

#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

extern "C" {

// Helper function to detect image format
bool is_jpeg(const unsigned char* data, size_t size) {
    return size >= 2 && data[0] == 0xFF && data[1] == 0xD8;
}

bool is_png(const unsigned char* data, size_t size) {
    return size >= 8 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47;
}

// Helper function to initialize ExifData with default values
void init_exif_data(ExifData& exif_data) {
    memset(exif_data.camera_make, 0, 64);
    memset(exif_data.camera_model, 0, 64);
    exif_data.software = nullptr;
    exif_data.iso_speed = 0;
    exif_data.shutter = 0.0;
    exif_data.aperture = 0.0;
    exif_data.focal_length = 0.0;
    exif_data.raw_width = 0;
    exif_data.raw_height = 0;
    exif_data.output_width = 0;
    exif_data.output_height = 0;
    exif_data.colors = 3; // Default to RGB
    exif_data.color_filter = 0;
    for (int i = 0; i < 4; i++) {
        exif_data.cam_mul[i] = 0.0;
    }
    exif_data.date_taken = nullptr;
    exif_data.lens = nullptr;
    exif_data.max_aperture = 0.0;
    exif_data.focal_length_35mm = 0;
    exif_data.description = nullptr;
    exif_data.artist = nullptr;
}

// Helper function to populate ExifData from TinyEXIF
void populate_exif_from_tinyexif(const TinyEXIF::EXIFInfo& exif_info, ExifData& exif_data) {
    // Copy camera make and model
    strncpy(exif_data.camera_make, exif_info.Make.c_str(), 63);
    exif_data.camera_make[63] = '\0';
    strncpy(exif_data.camera_model, exif_info.Model.c_str(), 63);
    exif_data.camera_model[63] = '\0';
    
    // Set other EXIF fields
    exif_data.iso_speed = exif_info.ISOSpeedRatings;
    exif_data.shutter = (exif_info.ExposureTime > 0) ? (1.0 / exif_info.ExposureTime) : 0.0;
    exif_data.aperture = exif_info.FNumber;
    exif_data.focal_length = exif_info.FocalLength;
    // Note: TinyEXIF may not have all fields, using available ones
    
    // Software is typically stored as a string, we'll need to manage memory carefully
    // For now, we'll leave it as nullptr to avoid memory management issues
}

int process_image_to_jpeg(const char* input_path, const char* output_path, ExifData& exif_data) {
    // Initialize EXIF data with defaults
    init_exif_data(exif_data);
    
    // Read input file
    std::ifstream file(input_path, std::ios::binary);
    if (!file) {
        std::cerr << "Failed to open input file: " << input_path << std::endl;
        return -1;
    }

    file.seekg(0, std::ios::end);
    size_t size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<unsigned char> input_data(size);
    file.read(reinterpret_cast<char*>(input_data.data()), size);
    file.close();

    if (size == 0) {
        std::cerr << "Empty input file: " << input_path << std::endl;
        return -1;
    }

    // Detect file format and handle accordingly
    if (is_jpeg(input_data.data(), size)) {
        // Handle JPEG files using TurboJPEG
        tjhandle decompress_handle = tjInitDecompress();
        if (!decompress_handle) {
            std::cerr << "Failed to initialize TurboJPEG decompressor" << std::endl;
            return -1;
        }

        int width, height, subsampling, colorspace;
        if (tjDecompressHeader3(decompress_handle, input_data.data(), size, &width, &height, &subsampling, &colorspace) != 0) {
            std::cerr << "Failed to read JPEG header: " << tjGetErrorStr() << std::endl;
            tjDestroy(decompress_handle);
            return -1;
        }

        // Set basic image dimensions in EXIF data
        exif_data.raw_width = width;
        exif_data.raw_height = height;
        exif_data.output_width = width;
        exif_data.output_height = height;

        // Decompress to RGB
        size_t rgb_buffer_size = width * height * tjPixelSize[TJPF_RGB];
        unsigned char* rgb_buffer = new unsigned char[rgb_buffer_size];

        if (tjDecompress2(decompress_handle, input_data.data(), size, rgb_buffer, width, 0, height, TJPF_RGB, TJFLAG_FASTDCT) != 0) {
            std::cerr << "Failed to decompress JPEG: " << tjGetErrorStr() << std::endl;
            delete[] rgb_buffer;
            tjDestroy(decompress_handle);
            return -1;
        }

        tjDestroy(decompress_handle);

        // Extract EXIF metadata from the original JPEG data
        TinyEXIF::EXIFInfo exif_info;
        if (exif_info.parseFromEXIFSegment(input_data.data(), size) == TinyEXIF::PARSE_SUCCESS) {
            std::cout << "EXIF Metadata found:" << std::endl;
            std::cout << "Camera Make: " << exif_info.Make << std::endl;
            std::cout << "Camera Model: " << exif_info.Model << std::endl;
            std::cout << "Focal Length: " << exif_info.FocalLength << "mm" << std::endl;
            std::cout << "ISO: " << exif_info.ISOSpeedRatings << std::endl;
            
            // Populate our ExifData structure
            populate_exif_from_tinyexif(exif_info, exif_data);
        } else {
            std::cout << "No EXIF metadata found in JPEG" << std::endl;
            // Set basic camera info for JPEG files without EXIF
            strncpy(exif_data.camera_make, "Unknown", 63);
            strncpy(exif_data.camera_model, "JPEG Image", 63);
        }

        // Re-compress as JPEG and save to output path
        tjhandle compress_handle = tjInitCompress();
        if (!compress_handle) {
            std::cerr << "Failed to initialize TurboJPEG compressor" << std::endl;
            delete[] rgb_buffer;
            return -1;
        }

        unsigned char* jpeg_buffer = nullptr;
        unsigned long jpeg_size = 0;
        
        if (tjCompress2(compress_handle, rgb_buffer, width, 0, height, TJPF_RGB, &jpeg_buffer, &jpeg_size, TJSAMP_444, 90, TJFLAG_FASTDCT) != 0) {
            std::cerr << "Failed to compress JPEG: " << tjGetErrorStr() << std::endl;
            delete[] rgb_buffer;
            tjDestroy(compress_handle);
            return -1;
        }

        // Write compressed JPEG to output file
        std::ofstream output_file(output_path, std::ios::binary);
        if (!output_file) {
            std::cerr << "Failed to open output file: " << output_path << std::endl;
            delete[] rgb_buffer;
            tjFree(jpeg_buffer);
            tjDestroy(compress_handle);
            return -1;
        }

        output_file.write(reinterpret_cast<char*>(jpeg_buffer), jpeg_size);
        output_file.close();

        // Cleanup
        delete[] rgb_buffer;
        tjFree(jpeg_buffer);
        tjDestroy(compress_handle);
        
        return 0;
    } else {
        // For non-JPEG files (PNG, TIFF, etc.), use stb_image to decode and convert to JPEG
        int width, height, channels;
        unsigned char* image_data = stbi_load(input_path, &width, &height, &channels, 3); // Force RGB (3 channels)
        
        if (!image_data) {
            std::cerr << "Failed to decode image with stb_image: " << stbi_failure_reason() << std::endl;
            return -1;
        }
        
        std::cout << "Decoded image: " << width << "x" << height << " with " << channels << " channels" << std::endl;
        
        // Set basic image dimensions and info in EXIF data
        exif_data.raw_width = width;
        exif_data.raw_height = height;
        exif_data.output_width = width;
        exif_data.output_height = height;
        exif_data.colors = 3; // RGB
        
        // Set basic camera info for non-JPEG files
        strncpy(exif_data.camera_make, "Unknown", 63);
        if (is_png(input_data.data(), size)) {
            strncpy(exif_data.camera_model, "PNG Image", 63);
        } else {
            strncpy(exif_data.camera_model, "Image File", 63);
        }
        
        // Compress the RGB data to JPEG using TurboJPEG
        tjhandle compress_handle = tjInitCompress();
        if (!compress_handle) {
            std::cerr << "Failed to initialize TurboJPEG compressor" << std::endl;
            stbi_image_free(image_data);
            return -1;
        }

        unsigned char* jpeg_buffer = nullptr;
        unsigned long jpeg_size = 0;
        
        if (tjCompress2(compress_handle, image_data, width, 0, height, TJPF_RGB, &jpeg_buffer, &jpeg_size, TJSAMP_444, 90, TJFLAG_FASTDCT) != 0) {
            std::cerr << "Failed to compress to JPEG: " << tjGetErrorStr() << std::endl;
            stbi_image_free(image_data);
            tjDestroy(compress_handle);
            return -1;
        }

        // Write compressed JPEG to output file
        std::ofstream output_file(output_path, std::ios::binary);
        if (!output_file) {
            std::cerr << "Failed to open output file: " << output_path << std::endl;
            stbi_image_free(image_data);
            tjFree(jpeg_buffer);
            tjDestroy(compress_handle);
            return -1;
        }

        output_file.write(reinterpret_cast<char*>(jpeg_buffer), jpeg_size);
        output_file.close();

        // Cleanup
        stbi_image_free(image_data);
        tjFree(jpeg_buffer);
        tjDestroy(compress_handle);
        
        std::cout << "Successfully converted to JPEG: " << width << "x" << height << std::endl;
        return 0;
    }
}

void free_buffer(unsigned char* buffer) {
    delete[] buffer;
}

}

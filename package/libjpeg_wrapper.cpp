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

// Helper function to populate ExifData from TinyEXIF (matching libraw_wrapper style)
void populate_exif_from_tinyexif(const TinyEXIF::EXIFInfo& exif_info, ExifData& exif_data) {
    // Copy camera make and model (like libraw_wrapper does)
    strncpy(exif_data.camera_make, exif_info.Make.c_str(), 63);
    exif_data.camera_make[63] = '\0';
    strncpy(exif_data.camera_model, exif_info.Model.c_str(), 63);
    exif_data.camera_model[63] = '\0';
    
    // Set EXIF fields (matching libraw_wrapper extraction pattern)
    exif_data.iso_speed = static_cast<int>(exif_info.ISOSpeedRatings);
    exif_data.shutter = exif_info.ExposureTime;
    exif_data.aperture = exif_info.FNumber;
    exif_data.focal_length = exif_info.FocalLength;
    exif_data.max_aperture = 0.0; // TinyEXIF doesn't have MaxApertureValue field
    exif_data.focal_length_35mm = 0; // TinyEXIF doesn't have FocalLengthIn35mm field
    
    // Note: Software, date_taken, lens, description, artist are typically stored as strings
    // For now, we'll leave them as nullptr to avoid memory management issues
    // This matches the pattern used in libraw_wrapper where these are pointer fields
    exif_data.software = nullptr;
    exif_data.date_taken = nullptr;
    exif_data.lens = nullptr;
    exif_data.description = nullptr;
    exif_data.artist = nullptr;
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

        // Extract EXIF metadata from the original JPEG data first
        TinyEXIF::EXIFInfo original_exif_info;
        bool has_original_exif = false;
        
        // Try to parse EXIF from the JPEG file directly
        std::ifstream exif_file(input_path, std::ios::binary);
        if (exif_file) {
            exif_file.seekg(0, std::ios::end);
            size_t file_size = exif_file.tellg();
            exif_file.seekg(0, std::ios::beg);
            
            std::vector<unsigned char> file_data(file_size);
            exif_file.read(reinterpret_cast<char*>(file_data.data()), file_size);
            exif_file.close();
            
            // Try parsing from JPEG file
            has_original_exif = (original_exif_info.parseFrom(file_data.data(), file_size) == TinyEXIF::PARSE_SUCCESS);
        }
        
        if (has_original_exif) {
            std::cout << "EXIF Metadata found in original:" << std::endl;
            std::cout << "Camera Make: " << original_exif_info.Make << std::endl;
            std::cout << "Camera Model: " << original_exif_info.Model << std::endl;
            std::cout << "Focal Length: " << original_exif_info.FocalLength << "mm" << std::endl;
            std::cout << "ISO: " << original_exif_info.ISOSpeedRatings << std::endl;
        } else {
            std::cout << "No EXIF data found in original JPEG" << std::endl;
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

        // Extract EXIF data from the final output JPEG file to ensure accuracy
        TinyEXIF::EXIFInfo final_exif_info;
        bool has_final_exif = (final_exif_info.parseFrom(jpeg_buffer, jpeg_size) == TinyEXIF::PARSE_SUCCESS);
        
        if (has_final_exif) {
            std::cout << "EXIF preserved in final output:" << std::endl;
            populate_exif_from_tinyexif(final_exif_info, exif_data);
        } else {
            // If original had EXIF but final doesn't, preserve original EXIF but update dimensions
            if (has_original_exif) {
                std::cout << "Using original EXIF with updated dimensions" << std::endl;
                populate_exif_from_tinyexif(original_exif_info, exif_data);
            } else {
                std::cout << "No EXIF data available, using basic info" << std::endl;
                strncpy(exif_data.camera_make, "Unknown", 63);
                exif_data.camera_make[63] = '\0';
                strncpy(exif_data.camera_model, "JPEG Image", 63);
                exif_data.camera_model[63] = '\0';
            }
        }

        // Always update dimensions to match final output (like libraw_wrapper does)
        exif_data.raw_width = width;
        exif_data.raw_height = height;
        exif_data.output_width = width;
        exif_data.output_height = height;
        exif_data.colors = 3; // RGB JPEG
        exif_data.color_filter = 0; // No color filter for processed JPEG
        
        // Initialize other fields like libraw_wrapper
        if (exif_data.iso_speed == 0) exif_data.iso_speed = 0;
        if (exif_data.shutter == 0.0) exif_data.shutter = 0.0;
        if (exif_data.aperture == 0.0) exif_data.aperture = 0.0;
        if (exif_data.focal_length == 0.0) exif_data.focal_length = 0.0;
        if (exif_data.max_aperture == 0.0) exif_data.max_aperture = 0.0;
        if (exif_data.focal_length_35mm == 0) exif_data.focal_length_35mm = 0;
        
        // Initialize camera multipliers to neutral values
        for (int i = 0; i < 4; i++) {
            if (exif_data.cam_mul[i] == 0.0) exif_data.cam_mul[i] = 1.0;
        }

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

        // Try to extract EXIF from the final output JPEG (though unlikely for converted files)
        TinyEXIF::EXIFInfo final_exif_info;
        if (final_exif_info.parseFromEXIFSegment(jpeg_buffer, jpeg_size) == TinyEXIF::PARSE_SUCCESS) {
            std::cout << "EXIF found in final JPEG output" << std::endl;
            populate_exif_from_tinyexif(final_exif_info, exif_data);
        } else {
            // Set metadata based on the final converted image properties (like libraw_wrapper)
            std::cout << "Setting metadata based on converted image properties" << std::endl;
            strncpy(exif_data.camera_make, "Unknown", 63);
            exif_data.camera_make[63] = '\0';
            if (is_png(input_data.data(), size)) {
                strncpy(exif_data.camera_model, "PNG->JPEG Conversion", 63);
            } else {
                strncpy(exif_data.camera_model, "Image->JPEG Conversion", 63);
            }
            exif_data.camera_model[63] = '\0';
        }

        // Always update dimensions to match final output (like libraw_wrapper does)
        exif_data.raw_width = width;
        exif_data.raw_height = height;
        exif_data.output_width = width;
        exif_data.output_height = height;
        exif_data.colors = 3; // RGB JPEG
        exif_data.color_filter = 0; // No color filter for processed JPEG
        
        // Initialize other fields like libraw_wrapper
        if (exif_data.iso_speed == 0) exif_data.iso_speed = 0;
        if (exif_data.shutter == 0.0) exif_data.shutter = 0.0;
        if (exif_data.aperture == 0.0) exif_data.aperture = 0.0;
        if (exif_data.focal_length == 0.0) exif_data.focal_length = 0.0;
        if (exif_data.max_aperture == 0.0) exif_data.max_aperture = 0.0;
        if (exif_data.focal_length_35mm == 0) exif_data.focal_length_35mm = 0;
        
        // Initialize camera multipliers to neutral values
        for (int i = 0; i < 4; i++) {
            if (exif_data.cam_mul[i] == 0.0) exif_data.cam_mul[i] = 1.0;
        }

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

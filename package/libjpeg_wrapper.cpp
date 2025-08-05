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

// Helper function to extract EXIF data from JPEG files
void extract_jpeg_exif(const std::vector<unsigned char>& input_data, ExifData& exif_data) {
    TinyEXIF::EXIFInfo original_exif_info;
    bool has_original_exif = (original_exif_info.parseFrom(input_data.data(), input_data.size()) == TinyEXIF::PARSE_SUCCESS);
    
    if (has_original_exif) {
        std::cout << "EXIF found in JPEG file" << std::endl;
        populate_exif_from_tinyexif(original_exif_info, exif_data);
    } else {
        std::cout << "No EXIF data available, using basic info" << std::endl;
        strncpy(exif_data.camera_make, "Unknown", 63);
        exif_data.camera_make[63] = '\0';
        strncpy(exif_data.camera_model, "JPEG Image", 63);
        exif_data.camera_model[63] = '\0';
    }
}

// Helper function to set default EXIF data for non-JPEG files
void extract_non_jpeg_exif(const std::vector<unsigned char>& input_data, ExifData& exif_data) {
    strncpy(exif_data.camera_make, "Unknown", 63);
    exif_data.camera_make[63] = '\0';
    
    if (is_png(input_data.data(), input_data.size())) {
        strncpy(exif_data.camera_model, "PNG->JPEG Conversion", 63);
    } else {
        strncpy(exif_data.camera_model, "Image->JPEG Conversion", 63);
    }
    exif_data.camera_model[63] = '\0';
}

// Helper function to finalize EXIF data with common values
void finalize_exif_data(ExifData& exif_data, int width, int height) {
    // Always update dimensions to match final output (like libraw_wrapper does)
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
}

// Helper function to save RGB data as JPEG
int save_rgb_as_jpeg(unsigned char* rgb_data, int width, int height, const char* output_path) {
    tjhandle compress_handle = tjInitCompress();
    if (!compress_handle) {
        std::cerr << "Failed to initialize TurboJPEG compressor" << std::endl;
        return -1;
    }

    unsigned char* jpeg_buffer = nullptr;
    unsigned long jpeg_size = 0;
    
    if (tjCompress2(compress_handle, rgb_data, width, 0, height, TJPF_RGB, &jpeg_buffer, &jpeg_size, TJSAMP_444, 75, TJFLAG_FASTDCT) != 0) {
        std::cerr << "Failed to compress JPEG: " << tjGetErrorStr() << std::endl;
        tjDestroy(compress_handle);
        return -1;
    }

    // Write compressed JPEG to output file
    std::ofstream output_file(output_path, std::ios::binary);
    if (!output_file) {
        std::cerr << "Failed to open output file: " << output_path << std::endl;
        tjFree(jpeg_buffer);
        tjDestroy(compress_handle);
        return -1;
    }

    output_file.write(reinterpret_cast<char*>(jpeg_buffer), jpeg_size);
    output_file.close();

    // Cleanup
    tjFree(jpeg_buffer);
    tjDestroy(compress_handle);
    
    return 0;
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

    // Decode image to RGB data
    int width, height;
    unsigned char* rgb_data = nullptr;
    
    if (is_jpeg(input_data.data(), size)) {
        // Handle JPEG files using TurboJPEG
        extract_jpeg_exif(input_data, exif_data);
        
        tjhandle decompress_handle = tjInitDecompress();
        if (!decompress_handle) {
            std::cerr << "Failed to initialize TurboJPEG decompressor" << std::endl;
            return -1;
        }

        int subsampling, colorspace;
        if (tjDecompressHeader3(decompress_handle, input_data.data(), size, &width, &height, &subsampling, &colorspace) != 0) {
            std::cerr << "Failed to read JPEG header: " << tjGetErrorStr() << std::endl;
            tjDestroy(decompress_handle);
            return -1;
        }

        // Store the original resolution in EXIF data
        exif_data.raw_width = width;
        exif_data.raw_height = height;

        // Decompress to RGB with quarter resolution
        width /= 2; // Adjust width for quarter resolution
        height /= 2; // Adjust height for quarter resolution
        size_t rgb_buffer_size = width * height * tjPixelSize[TJPF_RGB];
        rgb_data = new unsigned char[rgb_buffer_size];

        if (tjDecompress2(decompress_handle, input_data.data(), size, rgb_data, width, 0, height, TJPF_RGB, TJFLAG_FASTDCT) != 0) {
            std::cerr << "Failed to decompress JPEG: " << tjGetErrorStr() << std::endl;
            delete[] rgb_data;
            tjDestroy(decompress_handle);
            return -1;
        }

        tjDestroy(decompress_handle);

        // Rotate the image according to EXIF orientation
        TinyEXIF::EXIFInfo exif_info;
        exif_info.parseFrom(input_data.data(), input_data.size());
        int orientation = exif_info.Orientation;
        if (orientation > 1) {
            unsigned char* rotated_data = new unsigned char[rgb_buffer_size];
            // Only handle the most common orientations (1=normal, 3=180, 6=90 CW, 8=90 CCW)
            if (orientation == 3) {
                // 180 degree rotation
                for (int y = 0; y < height; ++y) {
                    for (int x = 0; x < width; ++x) {
                        int src_idx = (y * width + x) * 3;
                        int dst_idx = ((height - 1 - y) * width + (width - 1 - x)) * 3;
                        rotated_data[dst_idx] = rgb_data[src_idx];
                        rotated_data[dst_idx + 1] = rgb_data[src_idx + 1];
                        rotated_data[dst_idx + 2] = rgb_data[src_idx + 2];
                    }
                }
            } else if (orientation == 6) {
                // 90 degree CW rotation
                for (int y = 0; y < height; ++y) {
                    for (int x = 0; x < width; ++x) {
                        int src_idx = (y * width + x) * 3;
                        int dst_idx = (x * height + (height - 1 - y)) * 3;
                        rotated_data[dst_idx] = rgb_data[src_idx];
                        rotated_data[dst_idx + 1] = rgb_data[src_idx + 1];
                        rotated_data[dst_idx + 2] = rgb_data[src_idx + 2];
                    }
                }
                std::swap(width, height);
            } else if (orientation == 8) {
                // 90 degree CCW rotation
                for (int y = 0; y < height; ++y) {
                    for (int x = 0; x < width; ++x) {
                        int src_idx = (y * width + x) * 3;
                        int dst_idx = ((width - 1 - x) * height + y) * 3;
                        rotated_data[dst_idx] = rgb_data[src_idx];
                        rotated_data[dst_idx + 1] = rgb_data[src_idx + 1];
                        rotated_data[dst_idx + 2] = rgb_data[src_idx + 2];
                    }
                }
                std::swap(width, height);
            } else {
                // Orientation not handled, keep as is
                delete[] rotated_data;
                rotated_data = nullptr;
            }
            if (rotated_data) {
                delete[] rgb_data;
                rgb_data = rotated_data;
            }
        }
    } else {
        // For non-JPEG files (PNG, TIFF, etc.), use stb_image to decode
        extract_non_jpeg_exif(input_data, exif_data);

        int channels;
        rgb_data = stbi_load(input_path, &width, &height, &channels, 3); // Force RGB (3 channels)

        if (!rgb_data) {
            std::cerr << "Failed to decode image with stb_image: " << stbi_failure_reason() << std::endl;
            return -1;
        }

        // Store the original resolution in EXIF data
        exif_data.raw_width = width;
        exif_data.raw_height = height;

        // Downscale the image by a factor of two using nearest neighbor
        int new_width = width / 2;
        int new_height = height / 2;
        unsigned char* downscaled_data = new unsigned char[new_width * new_height * 3];

        for (int y = 0; y < new_height; ++y) {
            for (int x = 0; x < new_width; ++x) {
                int src_index = ((y * 2) * width + (x * 2)) * 3;
                int dst_index = (y * new_width + x) * 3;

                // Copy RGB values
                downscaled_data[dst_index] = rgb_data[src_index];
                downscaled_data[dst_index + 1] = rgb_data[src_index + 1];
                downscaled_data[dst_index + 2] = rgb_data[src_index + 2];
            }
        }

        // Free the original RGB data and replace it with the downscaled data
        stbi_image_free(rgb_data);
        rgb_data = downscaled_data;
        width = new_width;
        height = new_height;
    }

    // Finalize EXIF data with common values
    finalize_exif_data(exif_data, width, height);

    // Save RGB data as JPEG
    int result = save_rgb_as_jpeg(rgb_data, width, height, output_path);
    
    // Cleanup RGB data
    if (is_jpeg(input_data.data(), size)) {
        delete[] rgb_data;
    } else {
        stbi_image_free(rgb_data);
    }
    
    if (result == 0) {
        std::cout << "Successfully converted to JPEG: " << width << "x" << height << std::endl;
    }
    
    return result;
}

void free_buffer(unsigned char* buffer) {
    delete[] buffer;
}

}

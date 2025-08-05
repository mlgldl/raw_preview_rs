#include "libraw_wrapper.h"
#include "LibRaw/libraw.h"
#include "turbojpeg.h"
#include <string>
#include <cstring>
#include <cstdio>
#include <chrono>
#include <iostream>
#include <vector>
#include <tuple>
#include <fstream>

static std::string last_error;

extern "C" {

const char* get_last_error() {
    return last_error.c_str();
}

int convert_ppm_to_jpeg(const std::vector<unsigned char>& ppm_data, int width, int height, const char* jpeg_path, int quality) {
    tjhandle jpeg_compressor = tjInitCompress();
    if (!jpeg_compressor) {
        return RW_ERROR_PROCESS;
    }

    unsigned char* jpeg_buf = nullptr;
    unsigned long jpeg_size = 0;

    int result = tjCompress2(
        jpeg_compressor,
        ppm_data.data(),
        width,
        0, // pitch
        height,
        TJPF_RGB,
        &jpeg_buf,
        &jpeg_size,
        TJSAMP_444,
        quality,
        TJFLAG_FASTDCT
    );

    if (result != 0) {
        tjDestroy(jpeg_compressor);
        return RW_ERROR_PROCESS;
    }

    std::ofstream jpeg_file(jpeg_path, std::ios::binary);
    if (!jpeg_file.is_open()) {
        tjFree(jpeg_buf);
        tjDestroy(jpeg_compressor);
        return RW_ERROR_WRITE;
    }

    jpeg_file.write(reinterpret_cast<char*>(jpeg_buf), jpeg_size);
    jpeg_file.close();

    tjFree(jpeg_buf);
    tjDestroy(jpeg_compressor);

    return RW_SUCCESS;
}

int process_raw_to_ppm(const char* input_path, const char* output_path, ExifData& exif_data) {
    auto start_libraw = std::chrono::high_resolution_clock::now();

    last_error.clear();
    LibRaw* processor = new LibRaw();

    try {
        // Set processing parameters
        processor->imgdata.params.output_bps = 8; // 8 bits per channel for smaller files
        processor->imgdata.params.output_color = 1; // sRGB output
        processor->imgdata.params.use_camera_wb = 1; // Use camera white balance
        processor->imgdata.params.no_auto_bright = 1; // Disable auto brightness for speed
        processor->imgdata.params.use_camera_matrix = 1; // Use camera color matrix
        processor->imgdata.params.half_size = 1; // Reduce resolution to one quarter

        // Open the RAW file
        int ret = processor->open_file(input_path);
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to open file: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_OPEN_FILE;
        }

        // Unpack the RAW data
        ret = processor->unpack();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to unpack RAW data: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_UNPACK;
        }

        // Process the image
        ret = processor->dcraw_process();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to process image: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        auto end_libraw = std::chrono::high_resolution_clock::now();
        std::cout << "LibRaw processing time: "
                  << std::chrono::duration<double>(end_libraw - start_libraw).count()
                  << " seconds" << std::endl;

        // Get PPM data in memory
        libraw_processed_image_t* image = processor->dcraw_make_mem_image();
        if (!image) {
            last_error = "Failed to generate PPM data: ";
            last_error += libraw_strerror(LIBRAW_UNSPECIFIED_ERROR);
            delete processor;
            return RW_ERROR_WRITE;
        }

        if (image->type != LIBRAW_IMAGE_BITMAP || image->colors != 3 || image->bits != 8) {
            last_error = "Unsupported image format";
            LibRaw::dcraw_clear_mem(image);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        int width = image->width;
        int height = image->height;
        std::vector<unsigned char> ppm_data(image->data, image->data + (width * height * 3));

        LibRaw::dcraw_clear_mem(image);

        // Convert PPM to JPEG
        auto start_turbojpeg = std::chrono::high_resolution_clock::now();
        ret = convert_ppm_to_jpeg(ppm_data, width, height, output_path, 75); // Lower quality for smaller files
        auto end_turbojpeg = std::chrono::high_resolution_clock::now();

        if (ret != RW_SUCCESS) {
            last_error = "Failed to convert PPM to JPEG";
            delete processor;
            return ret;
        }

        std::cout << "TurboJPEG processing time: "
                  << std::chrono::duration<double>(end_turbojpeg - start_turbojpeg).count()
                  << " seconds" << std::endl;

        // Validate file size
        std::ifstream jpeg_file(output_path, std::ios::binary | std::ios::ate);
        if (jpeg_file.is_open()) {
            auto file_size = jpeg_file.tellg();
            jpeg_file.close();

            if (file_size > 2 * 1024 * 1024) { // Check if file size exceeds 2MB
                last_error = "JPEG file size exceeds 2MB";
                delete processor;
                return RW_ERROR_WRITE;
            }
        }

        delete processor;
        return RW_SUCCESS;

    } catch (const std::exception& e) {
        last_error = "Exception occurred: ";
        last_error += e.what();
        delete processor;
        return RW_ERROR_UNKNOWN;
    } catch (...) {
        last_error = "Unknown exception occurred";
        delete processor;
        return RW_ERROR_UNKNOWN;
    }
}

} // extern "C"

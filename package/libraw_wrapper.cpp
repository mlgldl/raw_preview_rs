#include "libraw_wrapper.h"
#include "libraw/libraw.h"
#include "turbojpeg.h"
#include <string>
#include <vector>
#include <fstream>
#include <algorithm>

// Global variable to store the last error message
static std::string last_error;

extern "C" {

/**
 * Retrieves the last error message that occurred during processing
 * @return Pointer to the error message string
 */

const char* get_last_error() {
    return last_error.c_str();
}

/**
 * Converts PPM image data to JPEG format and saves to file
 * @param ppm_data Vector containing RGB pixel data
 * @param width Image width in pixels
 * @param height Image height in pixels
 * @param jpeg_path Output file path for the JPEG
 * @param quality JPEG quality (1-100, higher is better quality)
 * @return RW_SUCCESS on success, error code on failure
 */
int convert_ppm_to_jpeg(const std::vector<unsigned char>& ppm_data, int width, int height, const char* jpeg_path, int quality) {
    // Initialize TurboJPEG compressor
    tjhandle jpeg_compressor = tjInitCompress();
    if (!jpeg_compressor) {
        return RW_ERROR_PROCESS;
    }

    unsigned char* jpeg_buf = nullptr;
    unsigned long jpeg_size = 0;

    // Compress RGB data to JPEG
    int result = tjCompress2(
        jpeg_compressor,
        ppm_data.data(),          // Source RGB data
        width,
        0,                        // Pitch (0 = width * pixel_size)
        height,
        TJPF_RGB,                 // Pixel format: RGB
        &jpeg_buf,                // Output buffer (allocated by TurboJPEG)
        &jpeg_size,               // Output size
        TJSAMP_444,               // Subsampling: no chroma subsampling
        quality,
        TJFLAG_FASTDCT            // Use fast DCT for speed
    );

    if (result != 0) {
        tjDestroy(jpeg_compressor);
        return RW_ERROR_PROCESS;
    }

    // Write JPEG data to file
    std::ofstream jpeg_file(jpeg_path, std::ios::binary);
    if (!jpeg_file.is_open()) {
        tjFree(jpeg_buf);
        tjDestroy(jpeg_compressor);
        return RW_ERROR_WRITE;
    }

    jpeg_file.write(reinterpret_cast<char*>(jpeg_buf), jpeg_size);
    jpeg_file.close();

    // Clean up TurboJPEG resources
    tjFree(jpeg_buf);
    tjDestroy(jpeg_compressor);

    return RW_SUCCESS;
}

/**
 * Processes a RAW image file and converts it to JPEG format
 * @param input_path Path to the input RAW file
 * @param output_path Path where the output JPEG will be saved
 * @param exif_data Reference to ExifData structure (currently unused but kept for API compatibility)
 * @return RW_SUCCESS on success, error code on failure
 */
int process_raw_to_jpeg(const char* input_path, const char* output_path, ExifData& exif_data) {
    // Suppress unused parameter warning - exif_data kept for future API compatibility
    (void)exif_data;
    
    last_error.clear();
    LibRaw* processor = new LibRaw();

    try {
        // Configure LibRaw processing parameters for fast preview generation
        processor->imgdata.params.output_bps = 8;        // 8 bits per channel for smaller files
        processor->imgdata.params.output_color = 1;      // sRGB output color space
        processor->imgdata.params.use_camera_wb = 1;     // Use camera white balance
        processor->imgdata.params.no_auto_bright = 1;    // Disable auto brightness for speed
        processor->imgdata.params.use_camera_matrix = 1; // Use camera color matrix
        processor->imgdata.params.half_size = 1;         // Reduce resolution to one quarter (for speed)
        
        // Raw processing options for better DNG compatibility with non-standard files
        processor->imgdata.rawparams.options = 0;        // Reset options
        processor->imgdata.rawparams.options |= 0x2000;  // Don't check DNG illuminant strictly
        processor->imgdata.rawparams.options |= 0x8000;  // DNG stage 2 processing
        processor->imgdata.rawparams.options |= 0x10000; // DNG stage 3 processing
        processor->imgdata.rawparams.options |= 0x40000; // Allow size changes during processing

        // Open and validate the RAW file
        int ret = processor->open_file(input_path);
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to open file: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_OPEN_FILE;
        }

        // Unpack the RAW sensor data
        ret = processor->unpack();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to unpack RAW data: ";
            last_error += libraw_strerror(ret);
            
            // Provide more helpful error message for DNG files
            std::string input_str(input_path);
            if (input_str.length() > 4) {
                std::string ext = input_str.substr(input_str.length() - 4);
                std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);
                if (ext == ".dng") {
                    last_error += " (Note: This may be a non-standard DNG file from a mobile device or unsupported DNG variant)";
                }
            }
            
            delete processor;
            return RW_ERROR_UNPACK;
        }

        // Process the RAW data (demosaicing, color correction, etc.)
        ret = processor->dcraw_process();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to process image: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        // Generate processed image data in memory
        libraw_processed_image_t* image = processor->dcraw_make_mem_image();
        if (!image) {
            last_error = "Failed to generate image data: ";
            last_error += libraw_strerror(LIBRAW_UNSPECIFIED_ERROR);
            delete processor;
            return RW_ERROR_WRITE;
        }

        // Validate that we got the expected image format (RGB bitmap)
        if (image->type != LIBRAW_IMAGE_BITMAP || image->colors != 3 || image->bits != 8) {
            last_error = "Unsupported image format";
            LibRaw::dcraw_clear_mem(image);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        // Extract image dimensions and pixel data
        int width = image->width;
        int height = image->height;
        std::vector<unsigned char> ppm_data(image->data, image->data + (width * height * 3));

        // Clean up LibRaw image data
        LibRaw::dcraw_clear_mem(image);

        // Convert the RGB data to JPEG format
        ret = convert_ppm_to_jpeg(ppm_data, width, height, output_path, 75); // 75% quality for balance of size/quality
        if (ret != RW_SUCCESS) {
            last_error = "Failed to convert to JPEG";
            delete processor;
            return ret;
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

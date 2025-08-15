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

        // Extract EXIF data from the processed image
        // Note: camera make/model are char arrays in LibRaw, not pointers
        strncpy((char*)exif_data.camera_make, processor->imgdata.idata.make, 63);
        ((char*)exif_data.camera_make)[63] = '\0';
        strncpy((char*)exif_data.camera_model, processor->imgdata.idata.model, 63);
        ((char*)exif_data.camera_model)[63] = '\0';
        
        exif_data.software = processor->imgdata.idata.software;
        exif_data.iso_speed = static_cast<int>(processor->imgdata.other.iso_speed);
        exif_data.shutter = processor->imgdata.other.shutter;
        exif_data.aperture = processor->imgdata.other.aperture;
        exif_data.focal_length = processor->imgdata.other.focal_len;
        exif_data.raw_width = processor->imgdata.sizes.raw_width;
        exif_data.raw_height = processor->imgdata.sizes.raw_height;
        exif_data.output_width = processor->imgdata.sizes.width;
        exif_data.output_height = processor->imgdata.sizes.height;
        exif_data.colors = processor->imgdata.idata.colors;
        exif_data.color_filter = static_cast<int>(processor->imgdata.idata.filters);
        
        // Copy camera multipliers
        for (int i = 0; i < 4; i++) {
            exif_data.cam_mul[i] = processor->imgdata.color.cam_mul[i];
        }
        
        exif_data.date_taken = processor->imgdata.other.desc;
        exif_data.lens = processor->imgdata.lens.Lens;
        exif_data.max_aperture = processor->imgdata.lens.EXIF_MaxAp;
        exif_data.focal_length_35mm = processor->imgdata.lens.FocalLengthIn35mmFormat;
        exif_data.description = processor->imgdata.other.desc;
        exif_data.artist = processor->imgdata.other.artist;

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

int process_raw_bytes_to_jpeg(const unsigned char* data, size_t size, const char* output_path, ExifData& exif_data) {
    (void)exif_data;
    last_error.clear();
    if (!data || size == 0) {
        last_error = "Empty input buffer";
        return RW_ERROR_OPEN_FILE;
    }

    LibRaw* processor = new LibRaw();
    try {
        processor->imgdata.params.output_bps = 8;
        processor->imgdata.params.output_color = 1;
        processor->imgdata.params.use_camera_wb = 1;
        processor->imgdata.params.no_auto_bright = 1;
        processor->imgdata.params.use_camera_matrix = 1;
        processor->imgdata.params.half_size = 1;

        processor->imgdata.rawparams.options = 0;
        processor->imgdata.rawparams.options |= 0x2000;
        processor->imgdata.rawparams.options |= 0x8000;
        processor->imgdata.rawparams.options |= 0x10000;
        processor->imgdata.rawparams.options |= 0x40000;

    // Use LibRaw's open_buffer API to read from memory
    int ret = processor->open_buffer(const_cast<unsigned char*>(data), (size_t)size);
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to open buffer: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_OPEN_FILE;
        }

        ret = processor->unpack();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to unpack RAW data: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_UNPACK;
        }

        ret = processor->dcraw_process();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to process image: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        strncpy((char*)exif_data.camera_make, processor->imgdata.idata.make, 63);
        ((char*)exif_data.camera_make)[63] = '\0';
        strncpy((char*)exif_data.camera_model, processor->imgdata.idata.model, 63);
        ((char*)exif_data.camera_model)[63] = '\0';

        exif_data.software = processor->imgdata.idata.software;
        exif_data.iso_speed = static_cast<int>(processor->imgdata.other.iso_speed);
        exif_data.shutter = processor->imgdata.other.shutter;
        exif_data.aperture = processor->imgdata.other.aperture;
        exif_data.focal_length = processor->imgdata.other.focal_len;
        exif_data.raw_width = processor->imgdata.sizes.raw_width;
        exif_data.raw_height = processor->imgdata.sizes.raw_height;
        exif_data.output_width = processor->imgdata.sizes.width;
        exif_data.output_height = processor->imgdata.sizes.height;
        exif_data.colors = processor->imgdata.idata.colors;
        exif_data.color_filter = static_cast<int>(processor->imgdata.idata.filters);

        for (int i = 0; i < 4; i++) {
            exif_data.cam_mul[i] = processor->imgdata.color.cam_mul[i];
        }

        exif_data.date_taken = processor->imgdata.other.desc;
        exif_data.lens = processor->imgdata.lens.Lens;
        exif_data.max_aperture = processor->imgdata.lens.EXIF_MaxAp;
        exif_data.focal_length_35mm = processor->imgdata.lens.FocalLengthIn35mmFormat;
        exif_data.description = processor->imgdata.other.desc;
        exif_data.artist = processor->imgdata.other.artist;

        libraw_processed_image_t* image = processor->dcraw_make_mem_image();
        if (!image) {
            last_error = "Failed to generate image data: ";
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

        ret = convert_ppm_to_jpeg(ppm_data, width, height, output_path, 75);
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

int process_raw_bytes_to_jpeg_buffer(const unsigned char* data, size_t size, unsigned char** out_buf, size_t* out_size, ExifData& exif_data) {
    if (!out_buf || !out_size) return RW_ERROR_UNKNOWN;
    *out_buf = nullptr;
    *out_size = 0;

    last_error.clear();
    if (!data || size == 0) {
        last_error = "Empty input buffer";
        return RW_ERROR_OPEN_FILE;
    }

    LibRaw* processor = new LibRaw();
    try {
        processor->imgdata.params.output_bps = 8;
        processor->imgdata.params.output_color = 1;
        processor->imgdata.params.use_camera_wb = 1;
        processor->imgdata.params.no_auto_bright = 1;
        processor->imgdata.params.use_camera_matrix = 1;
        processor->imgdata.params.half_size = 1;

        processor->imgdata.rawparams.options = 0;
        processor->imgdata.rawparams.options |= 0x2000;
        processor->imgdata.rawparams.options |= 0x8000;
        processor->imgdata.rawparams.options |= 0x10000;
        processor->imgdata.rawparams.options |= 0x40000;

        int ret = processor->open_buffer(const_cast<unsigned char*>(data), (size_t)size);
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to open buffer: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_OPEN_FILE;
        }

        ret = processor->unpack();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to unpack RAW data: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_UNPACK;
        }

        ret = processor->dcraw_process();
        if (ret != LIBRAW_SUCCESS) {
            last_error = "Failed to process image: ";
            last_error += libraw_strerror(ret);
            delete processor;
            return RW_ERROR_PROCESS;
        }

        strncpy((char*)exif_data.camera_make, processor->imgdata.idata.make, 63);
        ((char*)exif_data.camera_make)[63] = '\0';
        strncpy((char*)exif_data.camera_model, processor->imgdata.idata.model, 63);
        ((char*)exif_data.camera_model)[63] = '\0';

        exif_data.software = processor->imgdata.idata.software;
        exif_data.iso_speed = static_cast<int>(processor->imgdata.other.iso_speed);
        exif_data.shutter = processor->imgdata.other.shutter;
        exif_data.aperture = processor->imgdata.other.aperture;
        exif_data.focal_length = processor->imgdata.other.focal_len;
        exif_data.raw_width = processor->imgdata.sizes.raw_width;
        exif_data.raw_height = processor->imgdata.sizes.raw_height;
        exif_data.output_width = processor->imgdata.sizes.width;
        exif_data.output_height = processor->imgdata.sizes.height;
        exif_data.colors = processor->imgdata.idata.colors;
        exif_data.color_filter = static_cast<int>(processor->imgdata.idata.filters);

        for (int i = 0; i < 4; i++) exif_data.cam_mul[i] = processor->imgdata.color.cam_mul[i];

        exif_data.date_taken = processor->imgdata.other.desc;
        exif_data.lens = processor->imgdata.lens.Lens;
        exif_data.max_aperture = processor->imgdata.lens.EXIF_MaxAp;
        exif_data.focal_length_35mm = processor->imgdata.lens.FocalLengthIn35mmFormat;
        exif_data.description = processor->imgdata.other.desc;
        exif_data.artist = processor->imgdata.other.artist;

        libraw_processed_image_t* image = processor->dcraw_make_mem_image();
        if (!image) {
            last_error = "Failed to generate image data: ";
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

        // Compress to JPEG in memory using TurboJPEG
        tjhandle jpeg_compressor = tjInitCompress();
        if (!jpeg_compressor) { delete processor; return RW_ERROR_PROCESS; }

        unsigned char* jpeg_buf = nullptr;
        unsigned long jpeg_size = 0;
        int cres = tjCompress2(jpeg_compressor, ppm_data.data(), width, 0, height, TJPF_RGB, &jpeg_buf, &jpeg_size, TJSAMP_444, 75, TJFLAG_FASTDCT);
        if (cres != 0) { tjDestroy(jpeg_compressor); delete processor; return RW_ERROR_PROCESS; }

        // Copy to caller buffer
        unsigned char* out = new unsigned char[jpeg_size];
        memcpy(out, jpeg_buf, jpeg_size);
        *out_buf = out;
        *out_size = jpeg_size;

        tjFree(jpeg_buf);
        tjDestroy(jpeg_compressor);
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

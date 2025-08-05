#ifndef LIBRAW_WRAPPER_H
#define LIBRAW_WRAPPER_H

#include <string>
#include <vector>

#ifdef __cplusplus
extern "C" {
#endif

// Return codes
#define RW_SUCCESS 0
#define RW_ERROR_OPEN_FILE 1
#define RW_ERROR_UNPACK 2
#define RW_ERROR_PROCESS 3
#define RW_ERROR_WRITE 4
#define RW_ERROR_UNKNOWN 5

struct ExifData {
    char camera_make[64];
    char camera_model[64];
    char* software;
    int iso_speed;
    double shutter;
    double aperture;
    double focal_length;
    int raw_width;
    int raw_height;
    int output_width;
    int output_height;
    int colors;
    int color_filter;
    double cam_mul[4];
    char* date_taken;
    char* lens;
    double max_aperture;
    int focal_length_35mm;
    char* description;
    char* artist;
};

// Process RAW file to JPEG
// Returns 0 on success, error code on failure
int process_raw_to_jpeg(const char* input_path, const char* output_path, ExifData& exif_data);

// Convert PPM data in memory to JPEG
// quality ranges from 1 to 100, with 100 being the best quality
int convert_ppm_to_jpeg(const std::vector<unsigned char>& ppm_data, int width, int height, const char* jpeg_path, int quality);

// Get error message for the last error
const char* get_last_error();

#ifdef __cplusplus
}
#endif

#endif // LIBRAW_WRAPPER_H

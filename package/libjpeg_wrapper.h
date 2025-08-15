#ifndef LIBJPEG_WRAPPER_H
#define LIBJPEG_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

// Use the same ExifData structure as libraw_wrapper
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

// Process image file to JPEG with EXIF extraction
// Returns 0 on success, error code on failure
int process_image_to_jpeg(const char* input_path, const char* output_path, ExifData& exif_data);

// Process image data from memory (buffer) to JPEG with EXIF extraction
// `data` points to the input bytes and `size` is the length in bytes.
// Returns 0 on success, error code on failure
int process_image_bytes(const unsigned char* data, size_t size, const char* output_path, ExifData& exif_data);

void free_buffer(unsigned char* buffer);

// Process image data from memory and return JPEG bytes in a newly-allocated buffer.
// The caller receives `*out_buf` (allocated via new unsigned char[]) and `*out_size`.
// The caller must call `free_buffer` to release the returned buffer.
int process_image_bytes_to_buffer(const unsigned char* data, size_t size, unsigned char** out_buf, size_t* out_size, ExifData& exif_data);

#ifdef __cplusplus
}
#endif

#endif // LIBJPEG_WRAPPER_H

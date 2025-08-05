#ifndef LIBJPEG_WRAPPER_H
#define LIBJPEG_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

struct JpegInfo {
    int width;
    int height;
    int subsampling;
    int colorspace;
};

int decode_jpeg(const char* input_path, unsigned char** output_buffer, JpegInfo* info);
void free_buffer(unsigned char* buffer);

#ifdef __cplusplus
}
#endif

#endif // LIBJPEG_WRAPPER_H

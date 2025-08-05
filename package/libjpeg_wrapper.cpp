#include <turbojpeg.h>
#include <iostream>
#include <fstream>
#include <vector>
#include <cstring>

extern "C" {

struct JpegInfo {
    int width;
    int height;
    int subsampling;
    int colorspace;
};

int decode_jpeg(const char* input_path, unsigned char** output_buffer, JpegInfo* info) {
    tjhandle handle = tjInitDecompress();
    if (!handle) {
        std::cerr << "Failed to initialize TurboJPEG decompressor" << std::endl;
        return -1;
    }

    std::ifstream file(input_path, std::ios::binary);
    if (!file) {
        std::cerr << "Failed to open input file: " << input_path << std::endl;
        tjDestroy(handle);
        return -1;
    }

    file.seekg(0, std::ios::end);
    size_t size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<unsigned char> jpeg_data(size);
    file.read(reinterpret_cast<char*>(jpeg_data.data()), size);

    int width, height, subsampling, colorspace;
    if (tjDecompressHeader3(handle, jpeg_data.data(), size, &width, &height, &subsampling, &colorspace) != 0) {
        std::cerr << "Failed to read JPEG header: " << tjGetErrorStr() << std::endl;
        tjDestroy(handle);
        return -1;
    }

    size_t buffer_size = width * height * tjPixelSize[TJPF_RGB];
    *output_buffer = new unsigned char[buffer_size];

    if (tjDecompress2(handle, jpeg_data.data(), size, *output_buffer, width, 0, height, TJPF_RGB, TJFLAG_FASTDCT) != 0) {
        std::cerr << "Failed to decompress JPEG: " << tjGetErrorStr() << std::endl;
        delete[] *output_buffer;
        tjDestroy(handle);
        return -1;
    }

    info->width = width;
    info->height = height;
    info->subsampling = subsampling;
    info->colorspace = colorspace;

    tjDestroy(handle);
    return 0;
}

void free_buffer(unsigned char* buffer) {
    delete[] buffer;
}

}

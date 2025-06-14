#include <stb_image.h>
#include <stb_image_write.h>
#include "image_utils.h"
#include <stdlib.h>
#include <string.h>

Image* load_image(const char* filename) {
    Image* img = (Image*)malloc(sizeof(Image));
    if (!img) return NULL;

    img->data = stbi_load(filename, &img->width, &img->height, &img->channels, 0);
    if (!img->data) {
        free(img);
        return NULL;
    }

    return img;
}

int save_image(const Image* image, const char* filename, int quality) {
    const char* ext = strrchr(filename, '.');
    if (!ext) return 0; // 无扩展名
    
    if (strcmp(ext, ".jpg") == 0 || strcmp(ext, ".jpeg") == 0) {
        return stbi_write_jpg(filename, image->width, image->height, image->channels, image->data, quality);
    } else if (strcmp(ext, ".png") == 0) {
        return stbi_write_png(filename, image->width, image->height, image->channels, image->data, 0);
    }

    return 0; // 不支持的格式
}

void free_image(Image* image) {
    if (image) {
        if (image->data) {
            stbi_image_free(image->data);
        }
        free(image);
    }
}

int compress_image(const char* input_path, const char* output_path, CompressorConfig config) {
    // 加载图片
    Image* img = load_image(input_path);
    if (!img) {
        return 1; // 加载失败
    }

    // 保存图片（压缩质量已在config中设置）
    int result = save_image(img, output_path, config.quality);
    
    // 释放资源
    free_image(img);
    
    return result ? 0 : 1; // 转换返回值格式
}

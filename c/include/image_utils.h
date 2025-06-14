#ifndef IMAGE_UTILS_H
#define IMAGE_UTILS_H

#include <stdint.h>
#include "image_compressor.h"

typedef struct {
    uint8_t* data;      // 图片像素数据
    int width;          // 图片宽度
    int height;         // 图片高度
    int channels;       // 颜色通道数(3=RGB, 4=RGBA)
} Image;

/**
 * 从文件加载图片
 * @param filename 文件名
 * @return 加载的图片，失败返回NULL
 */
Image* load_image(const char* filename);

/**
 * 保存图片到文件
 * @param image 图片数据
 * @param filename 文件名
 * @param quality 压缩质量(1-100)
 * @return 0成功，非零失败
 */
int save_image(const Image* image, const char* filename, int quality);

/**
 * 释放图片内存
 * @param image 图片指针
 */
void free_image(Image* image);

/**
 * 压缩图片
 * @param input_path 输入文件路径
 * @param output_path 输出文件路径
 * @param config 压缩配置
 * @return 0成功，非零失败
 */
int compress_image(const char* input_path, const char* output_path, CompressorConfig config);

#endif // IMAGE_UTILS_H

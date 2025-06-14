#ifndef IMAGE_COMPRESSOR_H
#define IMAGE_COMPRESSOR_H

typedef enum {
    CLEAN,
    MEDIUM,
    STRONG
} CompressionLevel;

typedef struct {
    CompressionLevel level;
    int quality; // 压缩质量(0-100)
    int threads; // 使用的线程数
} CompressorConfig;

/**
 * 初始化压缩器配置
 */
CompressorConfig init_config(CompressionLevel level);

/**
 * 压缩单个图片文件
 * @param input_path 输入文件路径
 * @param output_path 输出文件路径
 * @param config 压缩配置
 * @return 0成功，非零失败
 */
int compress_image(const char* input_path, const char* output_path, CompressorConfig config);

/**
 * 压缩目录中的所有图片
 * @param input_dir 输入目录
 * @param output_dir 输出目录
 * @param config 压缩配置
 * @return 0成功，非零失败
 */
int compress_directory(const char* input_dir, const char* output_dir, CompressorConfig config);

#endif // IMAGE_COMPRESSOR_H

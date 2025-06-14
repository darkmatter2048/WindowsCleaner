#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/image_compressor.h"
#include "../include/file_utils.h"
#include "../include/image_utils.h"

int main(int argc, char *argv[]) {
    if (argc < 4) {
        printf("Usage: %s <mode> <input> <output>\n", argv[0]);
        printf("Modes: clean, medium, strong\n");
        return 1;
    }

    // 解析压缩级别
    CompressionLevel level;
    if (strcmp(argv[1], "clean") == 0) {
        level = CLEAN;
    } else if (strcmp(argv[1], "medium") == 0) {
        level = MEDIUM;
    } else if (strcmp(argv[1], "strong") == 0) {
        level = STRONG;
    } else {
        printf("Invalid compression level\n");
        return 1;
    }

    // 初始化配置
    CompressorConfig config = init_config(level);

    const char* input = argv[2];
    const char* output = argv[3];

    // 检查输入路径是否存在
    if (!is_file(input) && !is_directory(input)) {
        printf("Input path does not exist: %s\n", input);
        return 1;
    }

    int result = 0;
    // 根据输入类型调用不同的压缩函数
    if (is_file(input)) {
        printf("Compressing single file: %s\n", input);
        result = compress_image(input, output, config);
    } else if (is_directory(input)) {
        printf("Directory compression not implemented yet\n");
        result = 1;
    }

    if (result == 0) {
        printf("Compression completed successfully with level: %d\n", level);
    } else {
        printf("Compression failed\n");
    }

    return result;
}

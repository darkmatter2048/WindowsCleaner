#ifndef COMPRESSION_LEVELS_H
#define COMPRESSION_LEVELS_H

#include "image_compressor.h"

/**
 * 初始化压缩器配置
 * @param level 压缩级别
 * @return 配置结构体
 */
CompressorConfig init_config(CompressionLevel level);

#endif // COMPRESSION_LEVELS_H

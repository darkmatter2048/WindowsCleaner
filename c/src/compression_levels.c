#include "compression_levels.h"
#include <stdlib.h>

CompressorConfig init_config(CompressionLevel level) {
    CompressorConfig config;
    config.level = level;
    
    // 设置不同压缩级别的质量参数
    switch(level) {
        case CLEAN:
            config.quality = 80;
            break;
        case MEDIUM:
            config.quality = 60;
            break;
        case STRONG:
            config.quality = 40;
            break;
        default:
            config.quality = 80; // 默认高质量
    }

    // 设置默认线程数(简化处理，实际应该获取系统CPU核心数)
    config.threads = 4; 

    return config;
}

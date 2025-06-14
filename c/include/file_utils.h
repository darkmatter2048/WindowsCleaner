#ifndef FILE_UTILS_H
#define FILE_UTILS_H

#include <stdbool.h>

/**
 * 检查路径是否是文件
 * @param path 路径
 * @return true是文件，false不是文件
 */
bool is_file(const char* path);

/**
 * 检查路径是否是目录
 * @param path 路径
 * @return true是目录，false不是目录
 */
bool is_directory(const char* path);

/**
 * 获取文件扩展名
 * @param filename 文件名
 * @return 小写的扩展名(不带点)，NULL表示无扩展名
 */
const char* get_file_extension(const char* filename);

/**
 * 创建目录(包括父目录)
 * @param path 目录路径
 * @return 0成功，非零失败
 */
int create_directory(const char* path);

#endif // FILE_UTILS_H

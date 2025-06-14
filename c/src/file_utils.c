#include "file_utils.h"
#include <sys/stat.h>
#include <string.h>
#include <ctype.h>
#include <stdlib.h>

#ifdef _WIN32
#include <direct.h>
#define MKDIR(path) _mkdir(path)
#else
#include <sys/stat.h>
#include <sys/types.h>
#define MKDIR(path) mkdir(path, 0755)
#endif

bool is_file(const char* path) {
    struct stat path_stat;
    if (stat(path, &path_stat) != 0) {
        return false;
    }
    return S_ISREG(path_stat.st_mode);
}

bool is_directory(const char* path) {
    struct stat path_stat;
    if (stat(path, &path_stat) != 0) {
        return false;
    }
    return S_ISDIR(path_stat.st_mode);
}

const char* get_file_extension(const char* filename) {
    const char* dot = strrchr(filename, '.');
    if (!dot || dot == filename) {
        return NULL;
    }
    return dot + 1;
}

int create_directory(const char* path) {
    return MKDIR(path);
}

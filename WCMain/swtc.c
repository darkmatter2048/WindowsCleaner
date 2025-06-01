#include <windows.h>
#include <shlobj.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// 清理目录内容（保留目录结构）
void CleanDirectory(const char* path) {
    char searchPath[MAX_PATH];
    snprintf(searchPath, MAX_PATH, "%s\\*", path);
    
    WIN32_FIND_DATAA findData;
    HANDLE hFind = FindFirstFileA(searchPath, &findData);
    
    if (hFind == INVALID_HANDLE_VALUE) return;
    
    do {
        if (strcmp(findData.cFileName, ".") == 0 || 
            strcmp(findData.cFileName, "..") == 0) {
            continue;
        }
        
        char fullPath[MAX_PATH];
        snprintf(fullPath, MAX_PATH, "%s\\%s", path, findData.cFileName);
        
        if (findData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            // 递归删除子目录
            CleanDirectory(fullPath);
            RemoveDirectoryA(fullPath);
        } else {
            // 删除文件
            DeleteFileA(fullPath);
        }
    } while (FindNextFileA(hFind, &findData));
    
    FindClose(hFind);
}

// 清理指定程序的缓存
void CleanAppCache(const char* appName) {
    char path[MAX_PATH];
    const char* cacheDirs[] = {
        "Local", "LocalLow", "Roaming"
    };
    
    // 清理AppData中的缓存
    for (int i = 0; i < 3; i++) {
        if (SUCCEEDED(SHGetFolderPathA(NULL, 
            i == 0 ? CSIDL_LOCAL_APPDATA : 
            i == 1 ? CSIDL_LOCAL_APPDATA : CSIDL_APPDATA,
            NULL, 0, path))) {
            
            char appPath[MAX_PATH];
            snprintf(appPath, MAX_PATH, "%s\\%s", path, appName);
            
            if (GetFileAttributesA(appPath) != INVALID_FILE_ATTRIBUTES) {
                printf("Cleaning: %s\n", appPath);
                CleanDirectory(appPath);
            }
        }
    }
    
    // 清理ProgramData中的缓存
    if (SUCCEEDED(SHGetFolderPathA(NULL, CSIDL_COMMON_APPDATA, NULL, 0, path))) {
        char appPath[MAX_PATH];
        snprintf(appPath, MAX_PATH, "%s\\%s", path, appName);
        
        if (GetFileAttributesA(appPath) != INVALID_FILE_ATTRIBUTES) {
            printf("Cleaning: %s\n", appPath);
            CleanDirectory(appPath);
        }
    }
}

// 枚举已安装程序并清理缓存
void CleanAllAppsCache() {
    HKEY hKey;
    const char* uninstallKeys[] = {
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"
    };
    
    for (int k = 0; k < 2; k++) {
        if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, uninstallKeys[k], 0, KEY_READ, &hKey) != ERROR_SUCCESS) {
            continue;
        }
        
        char subKeyName[255];
        DWORD index = 0;
        
        while (RegEnumKeyA(hKey, index, subKeyName, sizeof(subKeyName)) == ERROR_SUCCESS) {
            HKEY hSubKey;
            if (RegOpenKeyExA(hKey, subKeyName, 0, KEY_READ, &hSubKey) == ERROR_SUCCESS) {
                char displayName[255];
                DWORD size = sizeof(displayName);
                
                if (RegQueryValueExA(hSubKey, "DisplayName", NULL, NULL, 
                                    (LPBYTE)displayName, &size) == ERROR_SUCCESS) {
                    CleanAppCache(displayName);
                }
                RegCloseKey(hSubKey);
            }
            index++;
        }
        RegCloseKey(hKey);
    }
}

int main(int argc, char* argv[]) {
    printf("Windows App Cache Cleaner\n");
    
    if (argc > 1 && strcmp(argv[1], "/all") == 0) {
        printf("Cleaning all applications cache...\n");
        CleanAllAppsCache();
    } else if (argc > 1) {
        printf("Cleaning cache for: %s\n", argv[1]);
        CleanAppCache(argv[1]);
    } else {
        printf("Usage:\n");
        printf("  CacheCleaner.exe <appname>  # Clean specific app\n");
        printf("  CacheCleaner.exe /all       # Clean all apps\n");
    }
    
    printf("Operation completed.\n");
    return 0;
}
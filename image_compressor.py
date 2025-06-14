import os
import subprocess
from PyQt5.QtCore import Qt
from qfluentwidgets import InfoBar, InfoBarPosition

class ImageCompressor:
    def __init__(self, parent):
        self.parent = parent
        self.script_directory = os.path.dirname(os.path.abspath(__file__))
        
    def compress(self, mode, input_path, output_dir=None):
        """执行图片压缩
        
        Args:
            mode (str): 压缩模式 (clean/medium/strong)
            input_path (str): 输入文件或目录路径
            output_dir (str): 输出目录(可选)
        Returns:
            bool: 压缩是否成功
        """
        compressor_path = os.path.join(self.script_directory, "WCMain", "image_compressor.exe")
        
        if not os.path.exists(compressor_path):
            self._show_message("错误", "图片压缩工具未找到")
            return False
            
        try:
            if os.path.isdir(input_path):
                # 处理目录
                for root, _, files in os.walk(input_path):
                    for file in files:
                        if file.lower().endswith(('.png', '.jpg', '.jpeg')):
                            input_file = os.path.join(root, file)
                            output_file = os.path.join(output_dir or root, file)
                            subprocess.run([compressor_path, mode, input_file, output_file])
            else:
                # 处理单个文件
                if output_dir:
                    output_file = os.path.join(output_dir, os.path.basename(input_path))
                else:
                    output_file = input_path
                    
                subprocess.run([compressor_path, mode, input_path, output_file])
            return True
        except Exception as e:
            self._show_message("错误", f"图片压缩失败: {str(e)}")
            return False
            
    def _show_message(self, title, content):
        """显示提示信息"""
        InfoBar.success(
            title=title,
            content=content,
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=2000,
            parent=self.parent
        )

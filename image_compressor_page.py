import os
import sys
from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QLabel, 
                          QPushButton, QFileDialog, QSpacerItem,
                          QSizePolicy)
from qfluentwidgets import (FluentIcon, ComboBox, PushButton, 
                          InfoBar, InfoBarPosition)

# 添加当前目录到Python路径
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from image_compressor import ImageCompressor

class ImageCompressorPage(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.parent = parent
        self.setObjectName("imageCompressor")
        self.compressor = ImageCompressor(parent)
        
        # 主布局
        self.mainLayout = QVBoxLayout(self)
        self.mainLayout.setContentsMargins(20, 20, 20, 20)
        self.mainLayout.setSpacing(15)
        
        # 创建居中容器
        self.centerContainer = QWidget(self)
        self.centerLayout = QVBoxLayout(self.centerContainer)
        self.centerLayout.setAlignment(Qt.AlignCenter)
        
        # 添加上部弹性空间
        self.mainLayout.addStretch(1)
        
        # 压缩模式选择
        self.modeLabel = QLabel("压缩模式:", self)
        self.modeLabel.setAlignment(Qt.AlignCenter)
        self.modeCombo = ComboBox(self)
        self.modeCombo.addItems(["轻量压缩", "中等压缩", "强力压缩"])
        self.modeCombo.setCurrentIndex(1)
        self.modeCombo.setFixedWidth(200)  # 固定宽度
        
        # 文件选择按钮
        self.fileButton = PushButton("选择图片", self, FluentIcon.PHOTO)
        self.fileButton.setFixedWidth(200)
        self.fileButton.clicked.connect(self.selectFile)
        
        # 目录选择按钮
        self.dirButton = PushButton("选择文件夹", self, FluentIcon.FOLDER)
        self.dirButton.setFixedWidth(200)
        self.dirButton.clicked.connect(self.selectDirectory)
        
        # 压缩按钮
        self.compressButton = PushButton("开始压缩", self, FluentIcon.SEND)
        self.compressButton.setFixedWidth(200)
        self.compressButton.clicked.connect(self.compressImages)
        
        # 添加控件到居中布局
        self.centerLayout.addWidget(self.modeLabel)
        self.centerLayout.addWidget(self.modeCombo)
        self.centerLayout.addSpacing(15)
        self.centerLayout.addWidget(self.fileButton)
        self.centerLayout.addSpacing(5)
        self.centerLayout.addWidget(self.dirButton)
        self.centerLayout.addSpacing(15)
        self.centerLayout.addWidget(self.compressButton)
        
        # 添加居中容器到主布局
        self.mainLayout.addWidget(self.centerContainer)
        self.mainLayout.addStretch(1)
        
    def selectFile(self):
        """选择单个图片文件"""
        path, _ = QFileDialog.getOpenFileName(
            self,
            "选择图片",
            "",
            "图片文件 (*.png *.jpg *.jpeg)"
        )
        if path:
            self.selectedPath = path
            self.isDirectory = False
            
    def selectDirectory(self):
        """选择图片目录"""
        path = QFileDialog.getExistingDirectory(
            self,
            "选择图片文件夹",
            ""
        )
        if path:
            self.selectedPath = path
            self.isDirectory = True
            
    def compressImages(self):
        """执行压缩"""
        if not hasattr(self, 'selectedPath'):
            self.compressor._show_message("提示", "请先选择图片或文件夹")
            return
            
        if not hasattr(self, 'isDirectory'):
            self.isDirectory = False
            
        mode_map = {
            0: "clean",
            1: "medium", 
            2: "strong"
        }
        mode = mode_map.get(self.modeCombo.currentIndex(), "medium")
        
        if self.compressor.compress(mode, self.selectedPath):
            self.compressor._show_message("成功", "图片压缩完成")
            
    def _show_message(self, title, content):
        """显示提示信息"""
        if hasattr(self.parent, 'window') and self.parent.window():
            InfoBar.success(
                title=title,
                content=content,
                orient=Qt.Horizontal,
                isClosable=True,
                position=InfoBarPosition.TOP,
                duration=2000,
                parent=self.parent.window()
            )

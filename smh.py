from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget, QLabel, QVBoxLayout, QHBoxLayout, QFileDialog# 新增布局和标签组件

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        self.all_target_folder_path = ""
        self.target_folder_path = ""
        self.origin_folder_path = ""

        self.pushButton.setIcon(FIF.PASTE)
        self.pushButton_3.setIcon(FIF.PASTE)
        self.pushButton_5.setIcon(FIF.CUT)
        self.pushButton_2.setIcon(FIF.SHARE)
        self.pushButton_4.setIcon(FIF.SHARE)

        self.widget_3.clicked.connect(self.RunAppDataCleaner)
        self.label_4.setUrl("https://adc.dyblog.online")
        self.pushButton.clicked.connect(self.select_all_target_folder)
        self.pushButton_3.clicked.connect(self.select_target_folder)
        self.pushButton_5.clicked.connect(self.select_origin_folder)
        self.pushButton_2.clicked.connect(self.Notice_bar)

    # 选择目标文件夹并显示路径
    def select_all_target_folder(self): 
        self.all_target_folder_path = QFileDialog.getExistingDirectory(self, "选择目标文件夹", self.all_target_folder_path)
        if self.all_target_folder_path:
            self.label_5.setText(self.all_target_folder_path)

    def select_target_folder(self): 
        self.target_folder_path = QFileDialog.getExistingDirectory(self, "选择目标文件夹", self.target_folder_path)
        if self.target_folder_path:
            self.label_8.setText(self.target_folder_path)
            
    def select_origin_folder(self): 
        self.origin_folder_path = QFileDialog.getExistingDirectory(self, "选择目标文件夹", self.origin_folder_path)
        if self.origin_folder_path:
            self.label_9.setText(self.origin_folder_path)

    def RunAppDataCleaner(self):
        try:
            exe_path = "WCMain/AppDataCleaner.exe"
            subprocess.Popen([exe_path])
        except Exception as e:
            self.RunAPIError_bar(error_message=str(e))    

    def success_bar(self):
        InfoBar.success(
            title="操作完成！",
            content="转移完成！",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
    def Notice_bar(self, message=""):
        InfoBar.warning(
            title="出错了！😭",
            content=f"{message}",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
    
    def RunAPIError_bar(self,error_message=""):
        InfoBar.warning(
            title="出错了！😭",
            content=f"{error_message}",
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
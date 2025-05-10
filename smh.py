from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget, QLabel, QVBoxLayout, QHBoxLayout  # 新增布局和标签组件

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        self.pushButton.setIcon(FIF.PASTE)
        self.pushButton_3.setIcon(FIF.PASTE)
        self.pushButton_5.setIcon(FIF.CUT)
        self.pushButton_2.setIcon(FIF.SHARE)
        self.pushButton_4.setIcon(FIF.SHARE)

    def PermissionError_bar(self):
        InfoBar.warning(
            title="权限不足",
            content="请以管理员身份运行程序",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
    def Notice_bar(self):
        InfoBar.warning(
            title="敬请期待",
            content="开发中，敬请期待",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
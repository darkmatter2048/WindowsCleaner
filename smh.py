from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess

# 扫描已安装的应用程序名称及其AppData文件夹路径
def scan_installed_apps():
    apps = []
    # 扫描Windows系统中的所有AppData文件夹
    for root, dirs, files in os.walk('C:\\Users'):
        for dir in dirs:
            if dir == 'AppData':
                app_name = os.path.basename(root)
                app_data_path = os.path.join(root, dir)
                apps.append((app_name, app_data_path))
    return apps

class ScanThread(QThread):
    operationCompleted = pyqtSignal(list)
    operationFailed_permissionError = pyqtSignal()

    def run(self):
        try:
            # 执行扫描并获取结果
            apps = scan_installed_apps()
            # 发射携带扫描结果的完成信号
            self.operationCompleted.emit(apps)
        except PermissionError as e:
            self.operationFailed_permissionError.emit()

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        self.pushButton.setIcon(FIF.SEARCH)

    def show(self):
        # 在ScollArea中显示扫描的结果
        pass
        
    def PermissionError_bar(self):
        InfoBar.warning(
            title="扫描失败",
            content="扫描意外中断",
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
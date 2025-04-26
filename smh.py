from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget, QLabel, QVBoxLayout, QHBoxLayout  # 新增布局和标签组件

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess

class ScanThread(QThread):
    operationCompleted = pyqtSignal(list)
    operationFailed_permissionError = pyqtSignal()
    operation_scanning = pyqtSignal(str)

    def run(self):
        try:
            apps = self.scan_installed_apps()
            self.operationCompleted.emit(apps)
        except PermissionError as e:
            self.operationFailed_permissionError.emit()

    def scan_installed_apps(self):
        apps = []
        for root, dirs, files in os.walk('C:\\Users'):
            if 'AppData' in dirs:
                user_name = os.path.basename(root)
                app_data_path = os.path.join(root, 'AppData')
                # 计算应用数据总大小
                total_size = self.calculate_appdata_size(app_data_path)
                apps.append((user_name, app_data_path, total_size))
                self.operation_scanning.emit(user_name)
        return apps
    
    def calculate_appdata_size(self, path):
        total = 0
        for root, dirs, files in os.walk(path):
            for f in files:
                fp = os.path.join(root, f)
                try:
                    total += os.path.getsize(fp)
                except PermissionError:
                    continue
        return total

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)
        
        self.pushButton.setIcon(FIF.SEARCH)
        self.pushButton.clicked.connect(self.Notice_bar)
        
        # 初始化扫描线程
        self.scan_thread = ScanThread()
        self.scan_thread.operationCompleted.connect(self.show)
        self.scan_thread.operationFailed_permissionError.connect(self.PermissionError_bar)

    def start_scan(self):
        self.pushButton.setEnabled(False)
        self.scan_thread.start()

    def show(self, apps):
        # 清空现有内容
        content = self.scrollAreaWidgetContents()
        content_layout = content.layout() or QVBoxLayout(content)
        
        while content_layout.count():
            item = content_layout.takeAt(0)
            if widget := item.widget():
                widget.deleteLater()
        
        # 显示扫描结果
        for app_name, app_path, size in apps:
            card = QWidget()
            layout = QHBoxLayout(card)
            
            # 添加应用信息
            layout.addWidget(QLabel(f"用户: {app_name}"))
            layout.addWidget(QLabel(f"路径: {app_path}"))
            layout.addWidget(QLabel(f"大小: {size/1024/1024:.2f} MB"))
            
            content_layout.addWidget(card)
        
        self.pushButton.setEnabled(True)

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
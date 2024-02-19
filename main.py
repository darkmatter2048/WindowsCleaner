import sys
from PyQt6.QtWidgets import QApplication, QMainWindow, QMessageBox
from ui import Ui_MainWindow
from PyQt6.QtCore import Qt,QPoint
from PyQt6.QtGui import QMouseEvent, QIcon
import os
import shutil
import psutil

class Cleaner(QMainWindow, Ui_MainWindow):
    def __init__(self):
        super().__init__()
        # 创建UI对象
        self.ui = Ui_MainWindow()
        # 设置UI
        self.ui.setupUi(self)
        self.setWindowFlag(Qt.WindowType.FramelessWindowHint, True)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground, True)
        #self.setWindowFlag(Qt.WindowType.WindowStaysOnTopHint, True)
        self.setMouseTracking(True)
        self.draggable = False
        self.offset = QPoint()
        self.setWindowTitle("Windows Cleaner")
        self.ui.pushButton.clicked.connect(self.boost)
        self.ui.pushButton_2.clicked.connect(self.clean)
        self.flag = False

    def mousePressEvent(self, event: QMouseEvent):
        if event.button() == Qt.MouseButton.LeftButton:
            self.draggable = True
            self.offset = event.pos()

    def mouseMoveEvent(self, event: QMouseEvent):
        if self.draggable:
            self.move(self.mapToGlobal(event.pos() - self.offset))

    def mouseReleaseEvent(self, event: QMouseEvent):
        if event.button() == Qt.MouseButton.LeftButton:
            self.draggable = False

    def get_v(self):
        # 获取C盘的使用情况
        usage = psutil.disk_usage('C:\\')  # 请注意在Windows上使用双反斜杠来表示路径

        # 获取已用空间（以MB为单位）
        used_space_mb = round(usage.used / (1024 ** 2), 2)  # 转换为MB
        print(f"已用空间: {used_space_mb} MB")
        return used_space_mb

    def boost(self):
        print("优化加速")
        self.ui.label_3.setText("优化加速中······")
        v0 = self.get_v()
        self.boost_prefetch('C:\\Windows\\Prefetch')
        self.clean_temp_folder()
        v1 = self.get_v()
        self.ui.label_3.setText(f"加速完成！\n清理出{format(v0 - v1,'.2f')}MB空间")

    def boost_prefetch(self, folder_path):
        # 获取管理员权限
        if not os.access(folder_path, os.W_OK):
            raise PermissionError("You don't have permission to delete files in this folder.")

        # 遍历文件夹并删除文件
        for filename in os.listdir(folder_path):
            file_path = os.path.join(folder_path, filename)
            try:
                if os.path.isfile(file_path):
                    os.remove(file_path)
                elif os.path.isdir(file_path):
                    shutil.rmtree(file_path)
                print(f"Deleted: {file_path}")
            except Exception as e:
                print(f"Failed to delete: {file_path}, Error: {e}")

    def clean_temp_folder(self):
        temp_folder = os.environ.get('TEMP')
        for root, dirs, files in os.walk(temp_folder):
            for file in files:
                try:
                    os.remove(os.path.join(root, file))
                    print("Delete successfully")
                except Exception as e:
                    print(f"Failed to delete {os.path.join(root, file)}: {e}")
        print("Temporary files cleaned successfully")

    def clean(self):
        print("深度清理")
        self.ui.label_3.setText("深度清理中······")
        v0 = self.get_v()
        self.boost_prefetch('C:\\Windows\\SoftwareDistribution\\Download')
        self.boost_prefetch('C:\\Windows\\Prefetch')
        self.clean_temp_folder()
        v1 = self.get_v()
        self.ui.label_3.setText(f"加速完成！\n清理出{format(v0 - v1, '.2f')}MB空间")
        # 执行磁盘清理命令
        messageBox = QMessageBox()
        #messageBox.setWindowFlag(Qt.WindowType.WindowStaysOnTopHint)  # 设置置顶
        messageBox.setWindowTitle('是否继续清理？')
        messageBox.setWindowIcon(QIcon('resource/clean.png'))
        messageBox.setText(f"是否使用'磁盘清理'工具继续清理？")
        messageBox.setStandardButtons(QMessageBox.StandardButton.Ok | QMessageBox.StandardButton.Cancel)
        response = messageBox.exec()
        if response == QMessageBox.StandardButton.Ok:
            print("You clicked OK")
            try:
                os.popen("cleanmgr")
            except Exception as e:
                print(f"error{e}")
        else:
            print("You clicked Cancel")

        messageBox = QMessageBox()
        #messageBox.setWindowFlag(Qt.WindowType.WindowStaysOnTopHint)  # 设置置顶
        messageBox.setWindowTitle('是否继续清理？')
        messageBox.setWindowIcon(QIcon('resource/clean.png'))
        messageBox.setText(f"是否从'存储'删除临时文件？\n注意：此操作可能并不适用于所有Windows版本。")
        messageBox.setStandardButtons(QMessageBox.StandardButton.Ok | QMessageBox.StandardButton.Cancel)
        response = messageBox.exec()
        if response == QMessageBox.StandardButton.Ok:
            print("You clicked OK")
            try:
                os.system('start ms-settings:storagesense')
            except Exception as e:
                print(f"error{e}")
        else:
            print("You clicked Cancel")


if __name__ == '__main__':
    app = QApplication(sys.argv)
    form = Cleaner()
    form.show()
    sys.exit(app.exec())
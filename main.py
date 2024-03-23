import sys
from PyQt6.QtWidgets import QApplication, QMainWindow, QMessageBox
from ui import Ui_MainWindow
from PyQt6.QtCore import Qt, QPoint, QThread, pyqtSignal
from PyQt6.QtGui import QMouseEvent, QIcon, QMovie
import os
import shutil
import psutil
import subprocess
import ctypes

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_as_admin(file_path):
    if is_admin():
        # 程序已经拥有管理员权限，继续执行
        print("Running as administrator")
        app = QApplication(sys.argv)
        form = Cleaner()
        form.show()
        sys.exit(app.exec())
    else:
        # 请求管理员权限
        ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, file_path, None, 1)


def get_v():
    # 获取C盘的使用情况
    usage = psutil.disk_usage('C:\\')  # 请注意在Windows上使用双反斜杠来表示路径

    # 获取已用空间（以MB为单位）
    used_space_mb = round(usage.used / (1024 ** 2), 2)  # 转换为MB
    print(f"已用空间: {used_space_mb} MB")
    return used_space_mb


def boost_main():
    boost_prefetch('C:\\Windows\\Prefetch')
    clean_temp_folder()
    clean_system_logs()
    clean_browser_cache()

def clean_main():
    boost_prefetch('C:\\Windows\\SoftwareDistribution\\Download')
    boost_prefetch('C:\\Windows\\Prefetch')
    clean_temp_folder()
    clean_system_logs()
    clean_browser_cache()
    delete_restore_points()

def boost_prefetch(folder_path):
    if os.path.exists(folder_path):
        print(f"The path {folder_path} exists.")
    else:
        print(f"The path {folder_path} does not exist.")
        return

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

def clean_temp_folder():
    temp_folder = os.environ.get('TEMP')
    boost_prefetch(temp_folder)

def clean_browser_cache():
    browser_cache_folders = {
        "Chrome": os.path.join(os.getenv("LOCALAPPDATA"), "Google\\Chrome\\User Data\\Default\\Cache"),
        "Firefox": os.path.join(os.getenv("APPDATA"), "Mozilla\\Firefox\\Profiles"),
        "Edge": os.path.join(os.getenv("LOCALAPPDATA"), "Microsoft\\Edge\\User Data\\Default\\Cache")
    }
    print(browser_cache_folders)

    for browser, folder in browser_cache_folders.items():
        print(f"清理 {browser} 浏览器缓存：{folder}")
        boost_prefetch(folder)

def clean_system_logs():
    log_folder = os.path.join(os.getenv("SystemRoot"), "Logs")
    print(f"清理系统日志文件夹：{log_folder}")
    boost_prefetch(log_folder)

# 删除所有还原点的函数
def delete_restore_points():
    try:
        print("清理旧的系统还原点...")
        subprocess.run("vssadmin Delete Shadows /all /quiet", shell=True, check=True)
        print("成功清理旧的系统还原点")
    except subprocess.CalledProcessError as e:
        print("error：", e.returncode)


class CleanThread(QThread):
    operationCompleted = pyqtSignal()

    def run(self):
        clean_main()
        self.operationCompleted.emit()

class BoostThread(QThread):
    operationCompleted = pyqtSignal()

    def run(self):
        boost_main()
        self.operationCompleted.emit()


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
        self.movie = QMovie('resource/loading.gif')
        self.ui.label_3.setScaledContents(True)  # 将标签内容自适应大小

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

    def boost(self):
        print("优化加速")
        self.v0 = get_v()

        self.ui.pushButton_2.setEnabled(False)
        self.ui.pushButton.setEnabled(False)

        self.ui.label_3.setMovie(self.movie)
        self.movie.start()

        self.thread = BoostThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.stop_animation)
        self.thread.start()

    def clean(self):
        print("深度清理")
        self.v0 = get_v()

        self.ui.pushButton_2.setEnabled(False)
        self.ui.pushButton.setEnabled(False)

        self.ui.label_3.setMovie(self.movie)
        self.movie.start()

        self.thread = CleanThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.after_clean)
        self.thread.start()

    def on_operation_completed(self):
        pass

    def after_clean(self):
        self.stop_animation()
        # 执行磁盘清理命令
        messageBox = QMessageBox()
        # messageBox.setWindowFlag(Qt.WindowType.WindowStaysOnTopHint)  # 设置置顶
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
        # messageBox.setWindowFlag(Qt.WindowType.WindowStaysOnTopHint)  # 设置置顶
        messageBox.setWindowTitle('是否继续清理？')
        messageBox.setWindowIcon(QIcon('resource/clean.png'))
        messageBox.setText(f"是否从'存储'删除临时文件？\n注意：此操作可能并不适用于所有Windows版本。")
        messageBox.setStandardButtons(QMessageBox.StandardButton.Ok | QMessageBox.StandardButton.Cancel)
        response = messageBox.exec()
        if response == QMessageBox.StandardButton.Ok:
            print("You clicked OK")
            try:
                os.popen('start ms-settings:storagesense')
            except Exception as e:
                print(f"error{e}")
        else:
            print("You clicked Cancel")

    def stop_animation(self):
        self.movie.stop()
        self.ui.label_3.clear()
        self.ui.pushButton_2.setEnabled(True)
        self.ui.pushButton.setEnabled(True)
        self.v1 = get_v()
        if int(self.v0-self.v1) > 1024:
            self.ui.label_3.setText(f"加速完成！\n清理出{format((self.v0 - self.v1)/1024, '.2f')}GB空间")
        else:
            self.ui.label_3.setText(f"加速完成！\n清理出{format(self.v0 - self.v1, '.2f')}MB空间")

if __name__ == '__main__':
    # 以管理员权限运行
    file_path = sys.argv[0]
    run_as_admin(file_path)
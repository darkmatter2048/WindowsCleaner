from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget

from qfluentwidgets import FluentIcon as FIF,InfoBar,InfoBarPosition, InfoBarIcon, TeachingTip, TeachingTipTailPosition, Dialog
from mian_ui_ui import Ui_Form

import json
import os
import shutil
import psutil
import subprocess
import ctypes
try:
    from plyer import notification
except Exception as e:
    print(f"发生错误：{e}")
from datetime import datetime
import time

import os

current_file = "清理内存和临时文件，减少电脑卡顿"
# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'

with open(settings_path, 'r') as f:
    settings_data = json.load(f)

def get_drive_info(drive_letter):
    partitions = psutil.disk_partitions()
    for part in partitions:
        if part.device.startswith(drive_letter):
            try:
                # 检查挂载点是否存在
                if os.path.exists(part.mountpoint):
                    usage = psutil.disk_usage(part.mountpoint)
                    return usage.total, usage.used, usage.free, usage.percent
                else:
                    print(f"挂载点 {part.mountpoint} 不存在")
            except PermissionError:
                print(f"权限错误: 无法访问挂载点 {part.mountpoint}")
            except Exception as e:
                print(f"获取磁盘使用情况时发生错误: {e}")

def WeatherLate(date_str):
    now = datetime.now()
    try:
        specified_date = datetime.strptime(date_str, '%Y-%m-%d')
        date_difference = now - specified_date
        days_difference = date_difference.days
        return days_difference
    except ValueError as e:
        print(f"日期格式错误: {e}")
        return None

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
    global settings_data
    boost_prefetch('C:\\Windows\\SoftwareDistribution\\Download')
    boost_prefetch('C:\\Windows\\Prefetch')
    boost_prefetch('C:\\Windows\\Temp')
    boost_prefetch('C:\\Windows\\System32\\LogFiles')
    boost_prefetch('C:\\Windows\\System32\\DriverStore\\FileRepository')
    clean_temp_folder()
    clean_system_logs()
    clean_application_cache()
    try:
        clean_browser_cache()
    except Exception as e:
        print("浏览器缓存清理失败")    
    delete_restore_points()
    clean_tmp_files()
    user_list = settings_data["includePath"]
    for path in user_list:
        boost_prefetch(path)

def clean_application_cache():
    # 获取当前用户的用户名
    username = os.getlogin()
    # 构建应用程序缓存目录的路径
    cache_dir = f'C:\\Users\\{username}\\AppData\\Local\\Packages'
    # 遍历缓存目录下的所有文件夹
    for root, dirs, files in os.walk(cache_dir):
        for dir in dirs:
            # 构建每个应用程序缓存文件夹的路径
            app_cache_dir = os.path.join(root, dir)
            # 如果文件夹名称以"Cache"或"cache"结尾，则删除该文件夹
            if dir.lower().endswith('cache'):
                try:
                    shutil.rmtree(app_cache_dir)
                    print(f"Deleted {app_cache_dir}")
                except Exception as e:
                    print(f"Failed to delete {app_cache_dir}. Reason: {e}")


def clean_tmp_files():
    global current_file
    # 定义要遍历的文件夹路径
    folder_path = "C:\\"  # C盘根目录
    # 获取管理员权限
    if not os.access(folder_path, os.W_OK):
        raise PermissionError("You don't have permission to delete files in this folder.")
    # 遍历文件夹及其子文件夹
    for root, dirs, files in os.walk(folder_path):
        for file in files:
            # 检查文件是否为.tmp文件
            if file.endswith(".tmp") or file.endswith(".cache"):
                # 获取文件的完整路径
                file_path = os.path.join(root, file)
                # 删除文件
                try:
                    current_file = file_path
                    os.remove(file_path)
                    print(f"已删除：{file_path}")
                except Exception as e:
                    print(f"Failed to delete: {file_path}, Error: {e}")    

def boost_prefetch(folder_path):
    global current_file
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
                current_file = file_path
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

class FlashThread(QThread):
    operationCompleted = pyqtSignal(int)

    def run(self):
        while True:
            memory_info = psutil.virtual_memory()
            memory = int(memory_info.percent)
            self.operationCompleted.emit(memory)
            time.sleep(2)

class clean_page(QWidget, Ui_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        # 设置流畅图标
        self.pushButton.setIcon(FIF.BROOM)

        # 设置进度环取值范围和当前值
        self.progressBar.setRange(0, 100)
        self.progressBar.setValue(30)
        self.progressBar.setStrokeWidth(13)

        # 显示进度环内文本
        self.progressBar.setTextVisible(True)

        self.pushButton.clicked.connect(self.boost)
        self.widget_2.clicked.connect(self.clean)   

        self.show_work()
        self.run_flash()

    def on_operation_completed(self):
        print("完成")

    def run_flash(self):
        self.thread = FlashThread()
        self.thread.operationCompleted.connect(self.update_ui)
        self.thread.finished.connect(self.on_operation_completed)
        self.thread.start()

    def update_ui(self, index):
        global current_file
        self.progressBar.setValue(index)   
        file_name = os.path.basename(current_file)
        self.label_3.setText(f"{file_name}")

    def boost(self):
        print("优化加速")
        self.v0 = get_v()
        self.pushButton.setEnabled(False)
        self.widget_2.setEnabled(False)
        self.info_bar()
        self.thread = BoostThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.after_clean)
        self.thread.start()

    def clean(self):
        print("深度清理")
        self.v0 = get_v()
        self.pushButton.setEnabled(False)
        self.widget_2.setEnabled(False)
        self.info_bar()
        self.thread = CleanThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.deep_after_clean)
        self.thread.start()

    def show_work(self):
        drive_letter = 'C:'
        info = get_drive_info(drive_letter)
        if info:
            total, used, free, percent = info
            #print(f"C 盘总空间: {total / (1024 ** 3):.2f} GB")
            #print(f"C 盘已用空间: {used / (1024 ** 3):.2f} GB")
            self.label_9.setText(f"{free / (1024 ** 3):.2f}GB可用，共{total / (1024 ** 3):.2f}GB")
            #print(f"C 盘使用百分比: {percent}%")
        else:
            print(f"找不到驱动器 {drive_letter}")

    def after_clean(self):
        global current_file
        current_file = "清理内存和临时文件，减少电脑卡顿"
        self.v1 = get_v()
        if int(self.v0-self.v1) > 1024:
            message = f"加速完成！\n清理出{format((self.v0 - self.v1)/1024, '.2f')}GB空间"
        else:
            message = f"加速完成！\n清理出{format(self.v0 - self.v1, '.2f')}MB空间"
        self.pushButton.setEnabled(True)
        self.widget_2.setEnabled(True)
        self.showTeachingTip(content=message)
        self.show_work()

    def deep_after_clean(self):
        global current_file
        current_file = "清理内存和临时文件，减少电脑卡顿"
        self.v1 = get_v()
        if int(self.v0-self.v1) > 1024:
            message = f"加速完成！\n清理出{format((self.v0 - self.v1)/1024, '.2f')}GB空间"
        else:
            message = f"加速完成！\n清理出{format(self.v0 - self.v1, '.2f')}MB空间"
        self.pushButton.setEnabled(True)
        self.widget_2.setEnabled(True)
        self.showTeachingTip(content=message)
        self.show_work()

        w = Dialog("是否继续清理？", "是否使用'磁盘清理'工具继续清理？", self)
        w.yesButton.setText(f"是(Y)")
        w.cancelButton.setText(f"否(C)")
        if w.exec():
            try:
                os.popen("cleanmgr")
            except Exception as e:
                print(f"error{e}")
        else:
            print("Cancel")

        w = Dialog("是否继续清理？", f"是否从'存储'删除临时文件？\n注意：此操作可能并不适用于所有Windows版本。", self)
        w.yesButton.setText(f"是(Y)")
        w.cancelButton.setText(f"否(C)")
        if w.exec():
            try:
                os.popen('start ms-settings:storagesense')
            except Exception as e:
                print(f"error{e}")
        else:
            print("Cancel")        
        
    def showTeachingTip(self, content="设置成功！重启软件后生效。"):
        TeachingTip.create(
            target=self.label_11,
            icon=InfoBarIcon.SUCCESS,
            title='Success',
            content=content,
            isClosable=True,
            tailPosition=TeachingTipTailPosition.TOP,
            duration=2000,
            parent=self
        )            

    def info_bar(self):
        InfoBar.success(
        title='success',
        content="清理中，请稍后...",
        orient=Qt.Horizontal,
        isClosable=True,
        position=InfoBarPosition.TOP,
        duration=2000,
        parent=self
        )
         
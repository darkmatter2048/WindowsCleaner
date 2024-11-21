from PyQt5.QtCore import Qt, pyqtSignal, QThread
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, TeachingTip, TeachingTipTailPosition
from auto_ui_ui import Ui_AutoClean

import json
import ast
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


def get_settings():
    # 获取当前脚本的目录
    script_directory = os.path.dirname(os.path.abspath(__file__))
    settings_path = f'{script_directory}\WCMain\settings.json'

    with open(settings_path, 'r') as f:
        settings_data = json.load(f)
    return settings_data

# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'
with open(settings_path, 'r') as f:
    settings_data = json.load(f)

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
    clean_temp_folder()
    clean_system_logs()
    clean_browser_cache()
    delete_restore_points()
    user_list = settings_data["includePath"]
    for path in user_list:
        boost_prefetch(path)

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
        check_disk()
        self.operationCompleted.emit()

def check_disk():
    global settings_data
    total, used, free = shutil.disk_usage("C:\\")
    disk_room = int(free / (1024**3))
    le = int(settings_data["AutoCleanRoom"])
    if disk_room < le:
        v0 = get_v()
        clean_main()
        v1 = get_v()
        if int(v0-v1) > 1024:
            message = f"加速完成！\n清理出{format((v0 - v1)/1024, '.2f')}GB空间"
        else:
            message = f"加速完成！\n清理出{format(v0 - v1, '.2f')}MB空间"
        title = 'Windows Cleaner 4.0'  # 弹窗的标题
        icon = r'WCMain\resource\imgs\icon.ico'  # 可选参数，传入ico图标文件的路径，显示在弹窗上
        timeout = 10  # 弹窗的显示时间，以秒（s）作为单位
        try:
            notification.notify(title=title, message=message, timeout=timeout, app_icon=icon)
        except Exception as e:
            print(f"发生错误：{e}")
    time.sleep(300)


class auto_page(QWidget, Ui_AutoClean):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)
        global settings_data

        self.settings_data = settings_data

        self.Enabled = None 
        self.textEdit.setPlainText(str(self.settings_data["includePath"]))

        if self.settings_data["AutoCleanEnabled"] == "False":
            self.Enabled = False
            self.spinBox.setValue(self.settings_data["AutoCleanTime"])
            self.spinBox_2.setValue(self.settings_data["AutoCleanRoom"])
            self.pushButton.setChecked(False)
            self.comboBox.setEnabled(False)
            self.spinBox.setEnabled(False)
            self.spinBox_2.setEnabled(False)
        else:
            self.pushButton.setChecked(True)
            if self.settings_data["AutoCleanMode"] == 0:
                print("定期清理")
                self.spinBox.setValue(self.settings_data["AutoCleanTime"])
                self.spinBox_2.setValue(self.settings_data["AutoCleanRoom"])
                self.spinBox.setEnabled(True)
                self.spinBox_2.setEnabled(False)
                result = WeatherLate(self.settings_data["LastCleanTime"])
                if result >= self.settings_data["AutoCleanTime"]:
                    self.clean()
            else:
                print("空间不足时清理")
                self.spinBox.setValue(self.settings_data["AutoCleanTime"])
                self.spinBox_2.setValue(self.settings_data["AutoCleanRoom"])
                self.spinBox.setEnabled(False)
                self.spinBox_2.setEnabled(True)
                self.cac()
            self.comboBox.setCurrentIndex(int(self.settings_data["AutoCleanMode"]))                

        self.pushButton.checkedChanged.connect(self.onCheckedChanged)
        self.comboBox.currentIndexChanged.connect(self.ModeChanged)
        self.spinBox.valueChanged.connect(self.spinBox_changed)
        self.spinBox_2.valueChanged.connect(self.spinBox_2_changed)
        self.textEdit.textChanged.connect(self.save_paths)
    
    def on_operation_completed(self):
        pass
    
    def cac(self):
        print("深度清理")
        self.v0 = get_v()
        self.thread = BoostThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.after_clean)
        self.thread.start()

    def clean(self):
        print("深度清理")
        self.v0 = get_v()
        self.thread = CleanThread()
        self.thread.operationCompleted.connect(self.on_operation_completed)
        self.thread.finished.connect(self.after_clean)
        self.thread.start()

    def after_clean(self):
        self.v1 = get_v()
        if int(self.v0-self.v1) > 1024:
            message = f"加速完成！\n清理出{format((self.v0 - self.v1)/1024, '.2f')}GB空间"
        else:
            message = f"加速完成！\n清理出{format(self.v0 - self.v1, '.2f')}MB空间"
        title = 'Windows Cleaner 4.0'  # 弹窗的标题
        icon = r'WCMain\resource\imgs\icon.ico'  # 可选参数，传入ico图标文件的路径，显示在弹窗上
        timeout = 10  # 弹窗的显示时间，以秒（s）作为单位
        try:
            notification.notify(title=title, message=message, timeout=timeout, app_icon=icon)
        except Exception as e:
            print(f"发生错误：{e}")
        if self.settings_data["AutoCleanMode"] == 0:
            now = datetime.now()
            formatted_date = now.strftime('%Y-%m-%d')  # 格式：年-月-日
            self.settings_data["LastCleanTime"] = formatted_date
            with open('WCMain/settings.json', 'w') as file:
                json.dump(self.settings_data, file, indent=4)

    def save_paths(self):
        self.settings_data = get_settings()
        print("changed")
        # 字符串表示的列表
        list_str = f"{self.textEdit.toPlainText()}"
        #list_str = list_str.replace('\\', '\\\\')
        # 使用 ast.literal_eval 将字符串转换为列表
        try:
            list_obj = ast.literal_eval(str(list_str))
        except:
            print("Eh...")
            return    
        self.settings_data["includePath"] = list_obj
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)

    def spinBox_changed(self):
        self.settings_data = get_settings()
        self.settings_data["AutoCleanTime"] = self.spinBox.value()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4) 

    def spinBox_2_changed(self):
        self.settings_data = get_settings()
        self.settings_data["AutoCleanRoom"] = self.spinBox_2.value()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)             

    def ModeChanged(self, index):
        self.settings_data = get_settings()
        if index == 0:
            print("定期清理")
            self.spinBox.setEnabled(True)
            self.spinBox_2.setEnabled(False)
            self.settings_data["AutoCleanMode"] = 0
            # Change AutoCleanTime
        else:
            print("空间不足时清理")
            self.spinBox.setEnabled(False)
            self.spinBox_2.setEnabled(True)
            self.settings_data["AutoCleanMode"] = 1
            # Change AutoCleanRoom
        # Save Mode
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)     

    def onCheckedChanged(self, isChecked: bool):
        self.settings_data = get_settings()
        if isChecked==True:
            self.settings_data["AutoCleanEnabled"] = "True"
            self.comboBox.setEnabled(True)
            if self.settings_data["AutoRunEnabled"] == "False":
                self.showTeachingTip()
            if self.settings_data["AutoCleanMode"] == 0:
                self.spinBox.setEnabled(True)
                self.spinBox_2.setEnabled(False)
            else:    
                self.spinBox.setEnabled(False)
                self.spinBox_2.setEnabled(True)
        else:    
            self.settings_data["AutoCleanEnabled"] = "False"
            self.comboBox.setEnabled(False)
            self.spinBox.setEnabled(False)
            self.spinBox_2.setEnabled(False)
        # Save isChecked 
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)        

    def showTeachingTip(self, content="最好将软件设置为开机自启"):
        TeachingTip.create(
            target=self.label_11,
            icon=InfoBarIcon.SUCCESS,
            title='注意',
            content=content,
            isClosable=True,
            tailPosition=TeachingTipTailPosition.TOP,
            duration=2000,
            parent=self
        )   

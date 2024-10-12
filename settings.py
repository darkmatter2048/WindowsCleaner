from PyQt5.QtCore import Qt, pyqtSignal, QTimer
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget, QSystemTrayIcon

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, TeachingTip, TeachingTipTailPosition, ColorDialog, setThemeColor
from settings_ui_ui import Settings_UI_Form

import json
import winreg
import os
import sys
import requests

# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'

with open(settings_path, 'r') as f:
    settings_data = json.load(f)


def add_to_startup():
    # 获取当前 Python 程序的路径
    executable_path = sys.executable
    program_name = os.path.splitext(os.path.basename(executable_path))[0]

    try:
        key = winreg.HKEY_CURRENT_USER
        sub_key = r'Software\Microsoft\Windows\CurrentVersion\Run'
        with winreg.OpenKey(key, sub_key, 0, winreg.KEY_SET_VALUE) as registry_key:
            winreg.SetValueEx(registry_key, program_name, 0, winreg.REG_SZ, executable_path)
        print(f'{program_name} has been added to startup.')
    except Exception as e:
        print(f'Error adding to startup: {e}')

def remove_from_startup():
    program_name = os.path.splitext(os.path.basename(sys.executable))[0]
    try:
        key = winreg.HKEY_CURRENT_USER
        sub_key = r'Software\Microsoft\Windows\CurrentVersion\Run'
        with winreg.OpenKey(key, sub_key, 0, winreg.KEY_SET_VALUE) as registry_key:
            winreg.DeleteValue(registry_key, program_name)
        print(f'{program_name} has been removed from startup.')
    except Exception as e:
        print(f'Error removing from startup: {e}')

class settings_page(QWidget, Settings_UI_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        global settings_data
        
        self.settings_data = settings_data

        # 加载settings
        self.comboBox_7.setCurrentIndex(int(self.settings_data['theme'])) 
        self.AutoRun_2.setChecked(bool(self.settings_data['AutoRunEnabled']=="True")) 
        self.comboBox_5.setCurrentIndex(int(self.settings_data['closeEvent'])) 
        self.comboBox_6.setCurrentIndex(int(self.settings_data['update']))

        # 更新settings
        self.comboBox_7.currentIndexChanged.connect(self.ThemeChanged)
        self.pushButton_2.clicked.connect(self.ChangeThemeColor)
        self.AutoRun_2.stateChanged.connect(self.AutoRun)
        self.comboBox_5.currentIndexChanged.connect(self.CloseEventChanged)
        self.comboBox_6.currentIndexChanged.connect(self.AutoUpdateChanged)
              
    def AutoUpdateChanged(self, index):
        self.settings_data['update'] = self.comboBox_6.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()

    def ThemeChanged(self, index):
        self.settings_data['theme'] = self.comboBox_7.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()

    def CloseEventChanged(self, index):
        self.settings_data['closeEvent'] = self.comboBox_5.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()    

    def ChangeThemeColor(self):
        current_color = QColor(self.settings_data['themeColor'])  
        w = ColorDialog(current_color, "Choose Background Color", enableAlpha=False, parent=self)
        c_color = None
        w.colorChanged.connect(lambda color: self.mf(color))
        w.exec()
        
    def mf(self, color):
        print(color.name())
        #self.label_11.setStyleSheet(f"background-color: {color.name()};") 
        self.settings_data['themeColor'] = color.name()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        setThemeColor(color.name())    
        #self.showTeachingTip()

    def AutoRun(self):
        if self.AutoRun_2.isChecked():
            print("setAutoRun")
            self.settings_data['AutoRunEnabled'] = "True"
            add_to_startup()
        else:
            print("No")
            self.settings_data['AutoRunEnabled'] = "False"
            remove_from_startup()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)    

    def showTeachingTip(self, content="设置成功！重启软件后生效。"):
        TeachingTip.create(
            target=self.label_6,
            icon=InfoBarIcon.SUCCESS,
            title='Success',
            content=content,
            isClosable=True,
            tailPosition=TeachingTipTailPosition.TOP,
            duration=2000,
            parent=self
        )        
    
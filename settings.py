from PyQt5.QtCore import Qt, pyqtSignal, QTimer, QTranslator
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QApplication,QWidget, QSystemTrayIcon

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, TeachingTip, TeachingTipTailPosition, ColorDialog, setThemeColor
from settings_ui_ui import Settings_UI_Form

import json
import os


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

class settings_page(QWidget, Settings_UI_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)
        self.trans = QTranslator(self)
        _app = QApplication.instance()
        _app.installTranslator(self.trans)
        self.trans.load('WCMain/resource/Languages/English/qm/settings.qm')
        self.retranslateUi(self)

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
        #self.AutoRun_2.stateChanged.connect(self.AutoRun)
        self.comboBox_5.currentIndexChanged.connect(self.CloseEventChanged)
        self.comboBox_6.currentIndexChanged.connect(self.AutoUpdateChanged)
              
    def AutoUpdateChanged(self, index):
        self.settings_data = get_settings()
        self.settings_data['update'] = self.comboBox_6.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()

    def ThemeChanged(self, index):
        self.settings_data = get_settings()
        self.settings_data['theme'] = self.comboBox_7.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()

    def CloseEventChanged(self, index):
        self.settings_data = get_settings()
        self.settings_data['closeEvent'] = self.comboBox_5.currentIndex()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        self.showTeachingTip()    

    def ChangeThemeColor(self):
        self.settings_data = get_settings()
        current_color = QColor(self.settings_data['themeColor'])  
        w = ColorDialog(current_color, "Choose Background Color", enableAlpha=False, parent=self)
        c_color = None
        w.colorChanged.connect(lambda color: self.mf(color))
        w.exec()
        
    def mf(self, color):
        self.settings_data = get_settings()
        print(color.name())
        #self.label_11.setStyleSheet(f"background-color: {color.name()};") 
        self.settings_data['themeColor'] = color.name()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)
        setThemeColor(color.name())    
        #self.showTeachingTip() 
  
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
    
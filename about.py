from PyQt5.QtCore import Qt, pyqtSignal, QTranslator
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QApplication, QWidget


from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, TeachingTip, TeachingTipTailPosition
from about_ui_ui import Ui_Form

import requests
import json
import os

# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'
with open(settings_path, 'r') as f:
    settings_data = json.load(f) 

def download_version():
    # URL 地址
    url = 'https://wc.dyblog.online/version.json'
    # 发送 GET 请求
    response = requests.get(url)
    # 检查请求是否成功
    if response.status_code == 200:
        # 解析 JSON 数据
        data = response.json()
        print('获取的 JSON 数据：')
        print(data)
        return data
    else:
        print(f'下载失败，状态码: {response.status_code}')

class about_page(QWidget, Ui_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)
        self.trans = QTranslator(self)
        _app = QApplication.instance()
        _app.installTranslator(self.trans)
        self.trans.load('WCMain/resource/Languages/English/qm/about.qm')
        self.retranslateUi(self)

        global settings_data
        self.settings_data = settings_data 

        self.pushButton.clicked.connect(self.update)
        self.label_5.setUrl('https://wc.dyblog.online') 
        self.label_4.setUrl('https://space.bilibili.com/1847808902?spm_id_from=333.1007.0.0')   
        self.label_6.setUrl('https://www.dyblog.online')

    def update(self):
        self.showTeachingTip("检查中，请稍后")
        info = download_version()
        if info["version"] > self.settings_data["version"]:
            self.showTeachingTip("有新版本！")
            os.popen("start https://pan.quark.cn/s/03e706cb753a")
        else:
            print("已经是最新版本")
            self.showTeachingTip("已经是最新版本")

    def showTeachingTip(self, content="有新版本！"):
        TeachingTip.create(
            target=self.pushButton,
            icon=InfoBarIcon.SUCCESS,
            title='Success',
            content=content,
            isClosable=True,
            tailPosition=TeachingTipTailPosition.BOTTOM,
            duration=2000,
            parent=self
        )                
    
from PyQt5.QtCore import Qt, pyqtSignal, QTranslator
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QApplication, QWidget


from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, TeachingTip, TeachingTipTailPosition
from about_ui_ui import Ui_Form

import urllib.request
import urllib.error

import json
import os
import logger

# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'
with open(settings_path, 'r') as f:
    settings_data = json.load(f) 

def download_version():
    # URL 地址
    url = 'https://wc.dyblog.online/version.json'
    try:
        # 使用urllib代替requests
        with urllib.request.urlopen(url) as response:
            if response.status == 200:
                # 读取并解析JSON数据
                data = json.loads(response.read().decode('utf-8'))
                print('获取的 JSON 数据：')
                print(data)
                #logger.info('获取的 JSON 数据：')
                #logger.info(str(data))
                return data
            else:
                print(f'下载失败，状态码: {response.status}')
                #logger.error(f'下载失败，状态码: {response.status}')
    except urllib.error.URLError as e:
        print(f'下载失败，错误: {e.reason}')
        #logger.error(f'下载失败，错误: {e.reason}')
    except Exception as e:
        print(f'下载失败，错误: {str(e)}')
        #logger.error(f'下载失败，错误: {str(e)}')


class about_page(QWidget, Ui_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        global settings_data
        self.settings_data = settings_data 

        self.trans = QTranslator(self)
        _app = QApplication.instance()
        _app.installTranslator(self.trans)
        path = f"WCMain\\resource\\Languages\\{str(self.settings_data['language'])}\\qm\\about.qm"
        self.trans.load(path)
        self.retranslateUi(self)

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
    
from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from senior_ui_ui import Ui_Form

import os
import subprocess

class senior_page(QWidget, Ui_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        self.checkBox.stateChanged.connect(self.v_memory)
        self.checkBox_2.stateChanged.connect(self.shut_sleep)
        self.pushButton.clicked.connect(self.process)

    def v_memory(self):
        if self.checkBox.isChecked() == True:
            self.warning()

    def shut_sleep(self):
        if self.checkBox_2.isChecked() == True:
            self.success_bar()      

    def process(self):
        print(self.checkBox_2.isChecked())
        if self.checkBox_2.isChecked() == True:
            try:
                command = "powercfg -h off"
                result = subprocess.run(["powershell", "-Command", command], capture_output=True, text=True, check=True)
                print("输出:", result.stdout)
                print("错误输出:", result.stderr)
                self.success_bar_2()
            except:
                print("error")  
                self.warning("权限不足，无法执行该操作") 
        else:
            try:
                command = "powercfg -h on"
                result = subprocess.run(["powershell", "-Command", command], capture_output=True, text=True, check=True)
                print("输出:", result.stdout)
                print("错误输出:", result.stderr)
                self.success_bar_2()
            except:
                print("error")
                self.warning("权限不足，无法执行该操作")                

    def warning(self, content="敬请期待..."):
        InfoBar.warning(
        title='WARNING',
        content=content,
        orient=Qt.Horizontal,
        isClosable=True,
        position=InfoBarPosition.TOP,
        duration=-1,    # 永不消失
        parent=self
        )

    def success_bar(self):
        InfoBar.success(
        title='Success',
        content="取消勾选后执行可恢复",
        orient=Qt.Horizontal,
        isClosable=True,
        position=InfoBarPosition.TOP,
        duration=2000,
        parent=self
        )

    def success_bar_2(self):
        InfoBar.success(
        title='Success',
        content="执行成功！",
        orient=Qt.Horizontal,
        isClosable=True,
        position=InfoBarPosition.TOP,
        duration=2000,
        parent=self
        )    
    
    
    
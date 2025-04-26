from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QWidget

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        self.pushButton.setIcon(FIF.SEARCH)


        
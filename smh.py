from PyQt5.QtCore import Qt, pyqtSignal, QThread, QTranslator
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QApplication, QWidget, QLabel, QVBoxLayout, QHBoxLayout, QFileDialog# 新增布局和标签组件


from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition
from smh_ui_ui import Ui_smh

import os
import subprocess
import shutil

# 获取当前用户的用户名，用于拼接AppData文件夹路径
def get_adpath():
    username = os.getlogin()
    # 拼接AppData文件夹路径
    appdata_path = os.path.join('C:\\Users', username, 'AppData')
    return appdata_path

class CutThread(QThread):
    operationCompleted = pyqtSignal()
    operationFailed_permissionError = pyqtSignal(str)
    
    def __init__(self, origin, target):
        super().__init__()
        self.origin_folder = origin
        self.target_folder = target

    def run(self):
        try:
            # 创建包含原文件夹名的目标路径
            target_mk_folder = os.path.join(self.target_folder, os.path.basename(self.origin_folder))
            
            # 修正后的复制调用（目标路径改为新建的子目录）
            shutil.copytree(self.origin_folder, target_mk_folder)
            
            subprocess.run(f'taskkill /F /IM "{os.path.basename(self.origin_folder)}"', 
                         shell=True)
            shutil.rmtree(self.origin_folder)
            command = f'mklink /d {self.origin_folder} {target_mk_folder}'
            result = os.popen(command)
            self.operationCompleted.emit()
            '''
            if result == 0:
                print("符号链接创建成功！")
                self.operationCompleted.emit()
            else:
                print("符号链接创建失败！")
                self.operationFailed_permissionError.emit("符号链接创建失败！")
                '''
        except subprocess.CalledProcessError as e:
            self.operationFailed_permissionError.emit(f"命令执行失败: {str(e)}")
            print(f"命令执行失败: {str(e)}")
        except PermissionError as e:
            self.operationFailed_permissionError.emit(f"权限不足: {str(e)}")
        except Exception as e:
            self.operationFailed_permissionError.emit(f"未知错误: {str(e)}")

class smh_page(QWidget, Ui_smh):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)
        self.trans = QTranslator(self)
        _app = QApplication.instance()
        _app.installTranslator(self.trans)
        self.trans.load('WCMain/resource/Languages/English/qm/smh.qm')
        self.retranslateUi(self)

        self.all_target_folder_path = ""
        self.target_folder_path = ""
        self.origin_folder_path = ""

        #self.pushButton.setIcon(FIF.PASTE)
        self.pushButton_3.setIcon(FIF.PASTE)
        self.pushButton_5.setIcon(FIF.CUT)
        #self.pushButton_2.setIcon(FIF.SHARE)
        self.pushButton_4.setIcon(FIF.SHARE)

        self.widget_3.clicked.connect(self.RunAppDataCleaner)
        self.label_4.setUrl("https://adc.dyblog.online")
        #self.pushButton.clicked.connect(self.select_all_target_folder)
        self.pushButton_3.clicked.connect(self.select_target_folder)
        self.pushButton_5.clicked.connect(self.select_origin_folder)
        #self.pushButton_2.clicked.connect(self.mode1)
        self.pushButton_4.clicked.connect(self.mode2)

    # 将目标文件夹复制到所选文件夹中
    def CutAndCreatMkLink(self, origin_folder, target_folder):
        if origin_folder=="Path" or target_folder=="Path":
            self.RunAPIError_bar("请选择文件夹！")
        elif origin_folder==target_folder:
            self.RunAPIError_bar("源文件夹和目标文件夹不能相同！")
        else: 
            #self.pushButton_2.setEnabled(False)
            self.pushButton_4.setEnabled(False)
            # 创建并启动线程
            self.cut_thread = CutThread(origin_folder, target_folder)
            self.cut_thread.operationCompleted.connect(self.finish)
            self.cut_thread.operationFailed_permissionError.connect(self.finish_error)
            self.cut_thread.start()
    def mode1(self):
        self.origin_folder_path = get_adpath()
        self.target_folder_path = self.label_5.text()
        # 将/替换为\\
        self.origin_folder_path = self.origin_folder_path.replace('/', '\\')
        self.target_folder_path = self.target_folder_path.replace('/', '\\')
        self.CutAndCreatMkLink(self.origin_folder_path, self.target_folder_path)    

    def mode2(self):
        self.origin_folder_path = self.label_9.text()
        self.target_folder_path = self.label_8.text()
        self.origin_folder_path = self.origin_folder_path.replace('/', '\\')
        self.target_folder_path = self.target_folder_path.replace('/', '\\')
        self.CutAndCreatMkLink(self.origin_folder_path, self.target_folder_path)        

    def finish(self):
        self.cut_thread.quit()
        self.cut_thread.wait()
        #self.pushButton_2.setEnabled(True)
        self.pushButton_4.setEnabled(True)
        self.success_bar()

    def finish_error(self, error_message):
        self.cut_thread.quit()
        self.cut_thread.wait()
        self.pushButton_2.setEnabled(True)
        self.pushButton_4.setEnabled(True)
        self.RunAPIError_bar(error_message=error_message)    

    # 选择目标文件夹并显示路径
    def select_all_target_folder(self): 
        self.all_target_folder_path = QFileDialog.getExistingDirectory(self, "选择目标文件夹", self.all_target_folder_path)
        if self.all_target_folder_path:
            self.label_5.setText(self.all_target_folder_path)

    def select_target_folder(self): 
        self.target_folder_path = QFileDialog.getExistingDirectory(self, "选择目标文件夹", self.target_folder_path)
        if self.target_folder_path:
            self.label_8.setText(self.target_folder_path)
            
    def select_origin_folder(self): 
        ad_path = get_adpath()
        self.origin_folder_path = QFileDialog.getExistingDirectory(self, "选择源文件夹", ad_path)
        if self.origin_folder_path:
            self.label_9.setText(self.origin_folder_path)

            

    def RunAppDataCleaner(self):
        try:
            exe_path = "WCMain/AppDataCleaner.exe"
            subprocess.Popen([exe_path])
        except Exception as e:
            self.RunAPIError_bar(error_message=str(e))    

    def success_bar(self):
        InfoBar.success(
            title="操作完成！",
            content="转移完成！",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
    def Notice_bar(self, message=""):
        InfoBar.warning(
            title="出错了！😭",
            content=f"{message}",  # 修改错误提示内容
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
    
    def RunAPIError_bar(self,error_message=""):
        InfoBar.warning(
            title="出错了！😭",
            content=f"{error_message}",
            orient=Qt.Horizontal,
            isClosable=True,
            position=InfoBarPosition.TOP,
            duration=5000,
            parent=self,
        )
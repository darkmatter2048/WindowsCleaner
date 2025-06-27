from PyQt5.QtCore import Qt, pyqtSignal, QTranslator
from PyQt5.QtGui import QPixmap, QPainter, QColor
from PyQt5.QtWidgets import QApplication, QWidget

from qfluentwidgets import FluentIcon as FIF, InfoBarIcon, InfoBar, InfoBarPosition, FluentTranslator
from senior_ui_ui import Ui_Form

import os
import subprocess
import json

script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = f'{script_directory}\WCMain\settings.json'
with open(settings_path, 'r') as f:
    settings_data = json.load(f)

class senior_page(QWidget, Ui_Form):
    def __init__(self, parent=None):
        super().__init__(parent=parent)
        self.setupUi(self)

        global settings_data
        self.settings_data = settings_data

        self.trans = QTranslator(self)
        _app = QApplication.instance()
        _app.installTranslator(self.trans)
        path = f"WCMain\\resource\\Languages\\{str(self.settings_data['language'])}\\qm\\senior.qm"
        self.trans.load(path)
        self.retranslateUi(self)

        self.checkBox.stateChanged.connect(self.v_memory)
        self.checkBox_2.stateChanged.connect(self.shut_sleep)
        self.pushButton.clicked.connect(self.process)

    def v_memory(self):
        if self.checkBox.isChecked() == True:
            self.success_bar()

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
        
        if self.checkBox.isChecked() == True:
            try:
                # 关闭所有驱动器的虚拟内存
                disable_all = "Get-WmiObject Win32_PageFileSetting | ForEach-Object { $_.Delete() }"
                subprocess.run(["powershell", "-Command", disable_all], check=True)
                
                # 获取除C盘外最大的磁盘
                get_disk = """
                $disks = Get-Volume | Where-Object { 
                    $_.DriveType -eq 'Fixed' -and 
                    $_.DriveLetter -ne 'C' -and 
                    $_.FileSystemType -eq 'NTFS'
                } | Sort-Object -Property Size -Descending | Select-Object -First 1
                $disks.DriveLetter + ':'
                """
                disk = subprocess.check_output(["powershell", "-Command", get_disk]).decode().strip()
                
                # 设置新的虚拟内存（初始大小为物理内存的1.5倍）
                set_vmem = f"""
                $computer = Get-WmiObject -Class Win32_ComputerSystem
                $phyMem = [math]::Round($computer.TotalPhysicalMemory / 1GB).ToString()
                $initial = [math]::Round($phyMem * 1.5).ToString()
                
                $pageFile = '{disk}\\pagefile.sys'
                Set-WMIInstance -Class Win32_PageFileSetting -Arguments @{{
                    Name = $pageFile; 
                    InitialSize = $initial; 
                    MaximumSize = $initial 
                }}
                """
                subprocess.run(["powershell", "-Command", set_vmem], check=True)
                # 刷新系统设置
                subprocess.run(
                    ["powershell", "-Command", "Clear-RecycleBin -Force"],
                    check=True
                )
                
                self.success_bar_2()
            except subprocess.CalledProcessError as e:
                #self.warning(f"操作失败: {str(e)}")
                print("error")
            except Exception as e:
                #self.warning(f"发生意外错误: {str(e)}")
                print("error")
        else:
            try:
                # 关闭所有分页文件
                disable_all = "Get-WmiObject Win32_PageFileSetting | ForEach-Object { $_.Delete() }"
                subprocess.run(["powershell", "-Command", disable_all], check=True)
                
                # 启用自动管理并设置C盘分页文件
                set_auto = """
    $computer = Get-WmiObject -Class Win32_ComputerSystem
    $computer.AutomaticManagedPagefile = $true
    $computer.Put()
    """
                subprocess.run(["powershell", "-Command", set_auto], check=True)
                
                # 强制刷新系统设置
                subprocess.run(
                    ["powershell", "-Command", "wmic computersystem where name=\"%computername%\" call reboot"],
                    check=True
                )
                self.success_bar_2()
            except subprocess.CalledProcessError as e:
                #self.warning(f"操作失败: {str(e)}")
                print("error")
            except Exception as e:
                #self.warning(f"发生意外错误: {str(e)}")
                print("error")
                    
     
    def warning(self, content="敬请期待！"):
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
    
    
    
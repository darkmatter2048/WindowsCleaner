import sys

from PyQt5.QtCore import Qt, QUrl, QPoint, QTimer
from PyQt5.QtGui import QIcon, QDesktopServices
from PyQt5.QtWidgets import QApplication, QWidget,QMainWindow, QAction, QMenu, QSystemTrayIcon, QStyle

from qfluentwidgets import FluentWindow, SplitFluentWindow, FluentIcon, NavigationAvatarWidget, NavigationItemPosition, setTheme, Theme, setThemeColor,Dialog
from clean import clean_page
from settings import settings_page
from senior import senior_page
from about import about_page
from support import support_page
from auto import auto_page

# 导入日志模块
from logger import get_logger

import json
import winreg
import requests
from datetime import datetime
try:
    from plyer import notification
except Exception as e:
    print(f"发生错误：{e}")
import os

# 获取日志记录器实例
logger = get_logger()

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
        logger.error(f"日期格式错误: {e}")
        return None

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
        logger.info('获取的 JSON 数据：', data)
        return data
    else:
        print(f'下载失败，状态码: {response.status_code}')
        logger.error(f'下载失败，状态码: {response.status_code}')

class Demo(SplitFluentWindow):

    def __init__(self):
        super().__init__()
        global settings_data
        self.settings_data = settings_data  
        
        self.checked = False
        self.update()    

        # 创建托盘图标
        self.tray_icon = QSystemTrayIcon(self)
        self.tray_icon.setIcon(QIcon(':/imgs/imgs/clean.png')) # QApplication.style().standardIcon(QStyle.SP_ComputerIcon)
        self.tray_icon.setVisible(True)

        # 创建托盘图标的右键菜单
        self.tray_menu = QMenu()
        self.show_action = QAction('显示主界面', self)
        self.quit_action = QAction('退出程序', self)
        self.show_action.triggered.connect(self.show)
        self.quit_action.triggered.connect(QApplication.instance().quit)
        self.tray_menu.addAction(self.show_action)
        self.tray_menu.addAction(self.quit_action)

        self.tray_icon.setContextMenu(self.tray_menu)

        # 连接托盘图标的左键点击事件
        self.tray_icon.activated.connect(self.icon_activated)
        
        # create sub interface
        self.cleanpage = clean_page(self)
        self.cleanpage.setObjectName("clean")  # 大坑啊，objectName不能重复
        self.settingspage = settings_page(self)
        self.settingspage.setObjectName("settings")
        self.seniorpage = senior_page(self)
        self.seniorpage.setObjectName("senior")
        self.aboutpage = about_page(self)
        self.aboutpage.setObjectName("about")
        self.supportpage = support_page(self)
        self.supportpage.setObjectName("support") 
        self.autopage = auto_page(self)
        self.autopage.setObjectName("AutoClean")
        
        self.titleBar.maxBtn.hide()
        self.titleBar.setDoubleClickEnabled(False)
        
        self.initNavigation()
        self.initWindow() 

    def update(self):
        try:
            if self.checked == True:
                return
            else:
                self.checked = True
                if self.settings_data['update'] == 1:
                    info = download_version()
                    if info["version"] > self.settings_data["version"]:
                        self.show_tooltip()
                    else:
                        print("已经是最新版本")
                        logger.debug("已经是最新版本")
                elif self.settings_data['update'] == 2:
                    result = WeatherLate(self.settings_data["AutoUpdate"])
                    if result > 7:
                        info = download_version()
                        if info["version"] > self.settings_data["version"]:
                            self.show_tooltip()
                        else:
                            print("已经是最新版本")
                            logger.debug("已经是最新版本")
                        now = datetime.now()
                        formatted_date = now.strftime('%Y-%m-%d')  # 格式：年-月-日
                        self.settings_data["AutoUpdate"] = formatted_date
                        with open('WCMain/settings.json', 'w') as file:
                            json.dump(self.settings_data, file, indent=4)    
                    else:
                        print("天数不足")
                        logger.debug("天数不足")
        except Exception as e:
            print(f"更新时发生错误: {e}")
            logger.error(f"更新时发生错误: {e}")
        '''
        if self.checked:
            return
        else:
            self.checked = True
            if self.settings_data['update'] == 1:
                info = download_version()
                if info["version"] > self.settings_data["version"]:
                    self.show_tooltip()
                else:
                    print("已经是最新版本")
            elif self.settings_data['update'] == 2:
                result = WeatherLate(self.settings_data["AutoUpdate"])
                if result > 7:
                    info = download_version()
                    if info["version"] > self.settings_data["version"]:
                        self.show_tooltip()
                    else:
                        print("已经是最新版本")
                    now = datetime.now()
                    formatted_date = now.strftime('%Y-%m-%d')  # 格式：年-月-日
                    self.settings_data["AutoUpdate"] = formatted_date
                    with open('WCMain/settings.json', 'w') as file:
                        json.dump(self.settings_data, file, indent=4)    
                else:
                    print("天数不足") 
        '''             
                     


    def show_tooltip(self):
        # 使用 QTimer 让提示信息在系统托盘图标上显示
        QTimer.singleShot(1000, lambda: self.tray_icon.showMessage(
            'Windows Cleaner 4.0',
            'WindowsCleaner有新版本啦,快去更新吧！',
            QSystemTrayIcon.Information,
            2000
        ))    
    
    def icon_activated(self, reason):
        if reason == QSystemTrayIcon.Trigger:
            self.show()  

    def closeEvent(self, e):
        if self.settings_data['closeEvent'] == 1:
            self.close()
        elif self.settings_data['closeEvent'] == 2:
            e.ignore()
            self.hide()
        else:
            w = Dialog("关闭", "请选择关闭模式", self)
            w.yesButton.setText("最小化到系统托盘")
            w.cancelButton.setText("退出程序")
            if w.exec():
                e.ignore()
                self.hide()
            else:
                self.close()
                          
    def initWindow(self):
        self.setFixedSize(396,550)   # 288+28,520
        self.setWindowFlags(self.windowFlags() | Qt.WindowStaysOnTopHint)
        self.setWindowTitle('WindowsCleaner v5.0')
        self.setWindowIcon(QIcon(':/imgs/imgs/clean.png'))
        # 窗口居右下
        rect = QApplication.desktop().availableGeometry()
        w, h = rect.width(), rect.height()
        self.move(w-self.width()-13,h-self.height()-13)
    
    def initNavigation(self):
        # 添加子界面   
        self.addSubInterface(self.cleanpage, FluentIcon.HOME, '优化加速')
        self.addSubInterface(self.seniorpage, FluentIcon.DEVELOPER_TOOLS, '高级')
        self.addSubInterface(self.autopage, FluentIcon.BROOM, '自动清理')
        self.addSubInterface(self.aboutpage, FluentIcon.INFO, '关于')
        self.addSubInterface(self.supportpage, FluentIcon.HEART, '支持')

        # 添加其它导航类
        self.navigationInterface.addItem(
            routeKey="GitHub",
            icon=FluentIcon.GITHUB,
            text="GitHub",
            onClick=self.visit_res,
            position=NavigationItemPosition.BOTTOM
        )
        self.addSubInterface(self.settingspage, FluentIcon.SETTING, '设置', position=NavigationItemPosition.BOTTOM)

        self.navigationInterface.setExpandWidth(280)

    def visit_res(self):
        QDesktopServices.openUrl(QUrl("https://github.com/darkmatter2048/WindowsCleaner"))

def load_settings():
    global settings_data
    if settings_data["theme"]==1:
        setTheme(Theme.DARK)
    elif settings_data["theme"] == 2:
        setTheme(Theme.AUTO)
    setThemeColor(settings_data["themeColor"]) # #009faa  
       

if __name__ == "__main__":
    QApplication.setHighDpiScaleFactorRoundingPolicy(Qt.HighDpiScaleFactorRoundingPolicy.PassThrough)
    QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
    QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)

    load_settings()

    app = QApplication(sys.argv)
    w = Demo()
    if settings_data["AutoRunEnabled"] != "True":  
        w.show()
    else:
        message = f"Windows Cleaner 4.0已启动！\n单击系统托盘图标进入主页。"
        title = 'Windows Cleaner 4.0'  # 弹窗的标题
        icon = r'WCMain\resource\imgs\icon.ico'  # 可选参数，传入ico图标文件的路径，显示在弹窗上
        timeout = 10  # 弹窗的显示时间，以秒（s）作为单位
        try:
            notification.notify(title=title, message=message, timeout=timeout, app_icon=icon)
        except Exception as e:
            print(f"发生错误：{e}")
            logger.error(f"发生错误：{e}")
    app.exec_()
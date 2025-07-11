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
from smh import smh_page

# 导入日志模块
from logger import get_logger

import json
import winreg
import urllib.request
import urllib.error

from datetime import datetime
try:
    from plyer import notification
except Exception as e:
    print(f"发生错误：{e}")
import os
import winreg
import win32api
import win32con
import ctypes

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

# 获取日志记录器实例
logger = get_logger()

# 获取当前脚本的目录
script_directory = os.path.dirname(os.path.abspath(__file__))
settings_path = os.path.join(script_directory, 'WCMain', 'settings.json')  # 修复反斜杠问题
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

# 开机自启
def add_to_startup():
    def zhao():
        location = "SOFTWARE\Microsoft\Windows\CurrentVersion\Run"
        # 获取注册表该位置的所有键值
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, location)
        i = 0
        while True:
            try:
                # 获取注册表对应位置的键和值
                # print(winreg.EnumValue(key, i)[0], winreg.EnumValue(key, i)[1])
                if winreg.EnumValue(key, i)[0] == os.path.basename(sys.argv[0]):
                    return True
                i += 1
            except OSError as error:
                # 一定要关闭这个键
                winreg.CloseKey(key)
                break

    flag = zhao()
    if flag:
        pass
    else:
        sys.setrecursionlimit(1000000)
        name = os.path.basename(sys.argv[0])
        path = os.getcwd() + '\\' + os.path.basename(sys.argv[0])
        key = win32api.RegOpenKey(win32con.HKEY_CURRENT_USER, "SOFTWARE\Microsoft\Windows\CurrentVersion\Run", 0,
                                  win32con.KEY_ALL_ACCESS)
        win32api.RegSetValueEx(key, name, 0, win32con.REG_SZ, path)
        win32api.RegCloseKey(key)

def remove_from_startup():
    # 获取当前程序名称
    name = os.path.basename(sys.argv[0])

    try:
        # 打开注册表项
        key = win32api.RegOpenKey(
            win32con.HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
            0,  # 保留参数必须为0
            win32con.KEY_ALL_ACCESS
        )

        # 尝试删除注册表值
        win32api.RegDeleteValue(key, name)
        win32api.RegCloseKey(key)
        print("成功移除开机启动项")
        logger.info("成功移除开机启动项")

    except Exception as e:
        print(f"移除启动项失败: {e}")
        logger.error(f"移除启动项失败: {e}")


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
        self.smhpge = smh_page(self)

        self.titleBar.maxBtn.hide()
        self.titleBar.setDoubleClickEnabled(False)

        self.initNavigation()
        self.initWindow()

        self.cleanpage.widget_5.clicked.connect(self.switch)
        self.settingspage.AutoRun_2.stateChanged.connect(self.AutoRun)
        self.cleanpage.widget_4.clicked.connect(self.switchToSmh)

    def switch(self):
        self.switchTo(self.seniorpage)

    def switchToSmh(self):
        self.switchTo(self.smhpge)

    def AutoRun(self):
        #self.settings_data = get_settings()
        if self.settingspage.AutoRun_2.isChecked():
            print("setAutoRun")
            self.settings_data['AutoRunEnabled'] = "True"
            add_to_startup()
        else:
            print("No")
            self.settings_data['AutoRunEnabled'] = "False"
            remove_from_startup()
        with open('WCMain/settings.json', 'w') as file:
            json.dump(self.settings_data, file, indent=4)

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

    def show_tooltip(self):
        # 使用 QTimer 让提示信息在系统托盘图标上显示
        QTimer.singleShot(1000, lambda: self.tray_icon.showMessage(
            'Windows Cleaner',
            'WindowsCleaner有新版本啦,快去更新吧！',
            QSystemTrayIcon.Information,
            2000
        ))

    def check_version(self,version1, version2):
        # 将版本号拆分为数字列表
        v1 = list(map(int, version1.split('.')))
        v2 = list(map(int, version2.split('.')))

        # 填充零使长度一致
        max_len = max(len(v1), len(v2))
        v1 += [0] * (max_len - len(v1))
        v2 += [0] * (max_len - len(v2))

        # 逐位比较
        for a, b in zip(v1, v2):
            if a > b:
                return 1
            elif a < b:
                return -1
        return 0

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
        self.setFixedSize(410,566)   # 288+28,520
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
        self.addSubInterface(self.smhpge, FluentIcon.MOVE, '数据迁移')
        self.addSubInterface(self.aboutpage, FluentIcon.INFO, '关于')
        self.addSubInterface(self.supportpage, FluentIcon.HEART, '支持')

        # 添加其它导航类
        self.navigationInterface.addItem(
            routeKey="Help",
            icon=FluentIcon.QUESTION,
            text="常见问题",
            onClick=self.visit_faq,
            position=NavigationItemPosition.BOTTOM
        )
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
    def visit_faq(self):
        QDesktopServices.openUrl(QUrl("https://dyblog.online/windowscleaner#faq"))

def load_settings():
    global settings_data
    if settings_data["theme"]==1:
        setTheme(Theme.DARK)
    elif settings_data["theme"] == 2:
        setTheme(Theme.AUTO)
    setThemeColor(settings_data["themeColor"]) # #009faa


if __name__ == "__main__":
    if is_admin():
        # Code of your program here
        QApplication.setHighDpiScaleFactorRoundingPolicy(Qt.HighDpiScaleFactorRoundingPolicy.PassThrough)
        QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
        QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)

        load_settings()

        app = QApplication(sys.argv)
        w = Demo()
        print("正在以管理员权限运行")
        if settings_data["AutoRunEnabled"] != "True":
            w.show()
        else:
            message = f"Windows Cleaner已启动！\n单击系统托盘图标进入主页。"
            title = 'Windows Cleaner 5.0'  # 弹窗的标题
            icon = r'WCMain\resource\imgs\icon.ico'  # 可选参数，传入ico图标文件的路径，显示在弹窗上
            timeout = 10  # 弹窗的显示时间，以秒（s）作为单位
            try:
                notification.notify(title=title, message=message, timeout=timeout, app_icon=icon)
            except Exception as e:
                print(f"发生错误：{e}")
                logger.error(f"发生错误：{e}")
        app.exec_()
    else:
        # Re-run the program with admin rights
        #ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, " ".join(sys.argv[1:]), None, 1)
        ctypes.windll.shell32.ShellExecuteW(None,"runas", sys.executable, __file__, None, 1)
    '''
    QApplication.setHighDpiScaleFactorRoundingPolicy(Qt.HighDpiScaleFactorRoundingPolicy.PassThrough)
    QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)
    QApplication.setAttribute(Qt.AA_UseHighDpiPixmaps)

    load_settings()

    app = QApplication(sys.argv)
    w = Demo()
    if settings_data["AutoRunEnabled"] != "True":
        w.show()
    else:
        message = f"Windows Cleaner已启动！\n单击系统托盘图标进入主页。"
        title = 'Windows Cleaner 5.0'  # 弹窗的标题
        icon = r'WCMain\resource\imgs\icon.ico'  # 可选参数，传入ico图标文件的路径，显示在弹窗上
        timeout = 10  # 弹窗的显示时间，以秒（s）作为单位
        try:
            notification.notify(title=title, message=message, timeout=timeout, app_icon=icon)
        except Exception as e:
            print(f"发生错误：{e}")
            logger.error(f"发生错误：{e}")
    app.exec_()
    '''

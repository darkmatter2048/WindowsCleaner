<div align=center>
<img src="logo.png" width="150" height="150">
<h1>Windows Cleaner</h1>
<h3>专治C盘爆红及各种不服！</h3>
完全开源免费的C盘清理工具
</div>

## 🎨运行截图 GUI
| ![show1](readme/s_light.png) | ![show2](readme/s_dark.png) |
|:----------------------:|:----------------------:|

## 🖥系统要求 System Requirements

- Windows操作系统：Windows 10 或以上
- Windows: Windows 10 or above

## 使用方法

### 下载安装包

[Windows Cleaner官网：https://wc.dyblog.online](https://wc.dyblog.online)

或从[蓝奏云网盘](https://wwt.lanzn.com/b03xje5uf)下载Windows Cleaner(amd64)的安装包。

密码:4ar1

### 安装
一路Next即可，如果想以后方便打开可以勾选上`创建桌面快捷方式`选项。

### 手动编译
#### 源码运行
- 克隆此仓库
- 安装 Python 3.8
- 安装依赖`pip install -r requirements.txt`
- 运行`python main.py`
#### 本地编译
- 先完成源码运行
- 安装 Visual Studio 以及 msvc 编译器
- 安装 Nuitka
```pip
pip install nuitka
```
- 编译
```python
python -m nuitka --standalone --remove-output --windows-console-mode=“disable” --enable-plugins=“pyqt5” --output-dir=“dist” --main=“main.py” --windows-icon-from-ico=“icon.ico”
```
> [!tip]
>
> 如果您的电脑未安装 Visual Studio 以及 msvc 编译器，Nuitka 会直接从 Github 下载 Mingw64，不论电脑上是否安装 Mingw64！

##### 编译安装包
1. 电脑安装 Inno Setup
2. 使用 Inno Setup 打开`scipt.iss`，点击编译即可
3. 生成的安装程序在`releases`目录下

- 将`WCMain`文件夹复制到`dist\main.dist`下，运行`main.exe`即可
#### GitHub Actions（推荐）
- 全自动编译，直接运行（或勾选“生成安装包”生成安装程序），运行结束后下载编译产物全部解压即可使用(注：编译时间非常长，大概编译一次需要20-30分钟）/或下载带`Setup`字样的压缩包，解压后运行安装程序安装即可

### [Bilibili:Mr_Jacek](https://space.bilibili.com/1847808902?spm_id_from=333.1007.0.0)

## 🤝支持 Windows Cleaner的开发

[<img src="https://wc.dyblog.online/images/d.png" alt="Develop Image" style="width: 200px;"/>](https://dyblog.online/donate)

## 🎖 贡献者

<a href="https://github.com/darkmatter2048/WindowsCleaner/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=darkmatter2048/WindowsCleaner" />
</a>

## 星标历史

<a href="https://star-history.com/#darkmatter2048/WindowsCleaner&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=darkmatter2048/WindowsCleaner&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=darkmatter2048/WindowsCleaner&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=darkmatter2048/WindowsCleaner&type=Date" />
 </picture>
</a>

## Copyright & License ⚖

Copyright © 2021.DaYe 

<p xmlns:cc="http://creativecommons.org/ns#" xmlns:dct="http://purl.org/dc/terms/"><a property="dct:title" rel="cc:attributionURL" href=#>Windows Cleaner</a> by <a rel="cc:attributionURL dct:creator" property="cc:attributionName" href="https://www.dyblog.online/">DaYe</a> is licensed under <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/?ref=chooser-v1" target="_blank" rel="license noopener noreferrer" style="display:inline-block;">CC BY-NC-SA 4.0<img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/cc.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/by.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/nc.svg?ref=chooser-v1" alt=""><img style="height:22px!important;margin-left:3px;vertical-align:text-bottom;" src="https://mirrors.creativecommons.org/presskit/icons/sa.svg?ref=chooser-v1" alt=""></a></p>

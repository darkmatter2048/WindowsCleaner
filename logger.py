import logging
import os

# 创建日志目录
LOG_DIR = "logs"
if not os.path.exists(LOG_DIR):
    os.makedirs(LOG_DIR)

# 日志文件路径
LOG_FILE = os.path.join(LOG_DIR, "app.log")

# 配置日志
def setup_logger():
    # 创建一个logger
    logger = logging.getLogger("WCLog") 
    logger.setLevel(logging.DEBUG)  # 设置日志级别为DEBUG

    # 检查日志文件是否存在，如果存在则清空它
    if os.path.exists(LOG_FILE):
        with open(LOG_FILE, 'w'):
            pass

    # 创建一个handler，用于写入日志文件
    file_handler = logging.FileHandler(LOG_FILE)
    file_handler.setLevel(logging.DEBUG)  # 设置文件handler的日志级别为DEBUG

    # 定义handler的输出格式
    formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    file_handler.setFormatter(formatter)

    # 给logger添加handler
    logger.addHandler(file_handler)

    return logger


# 获取日志记录器实例
logger = setup_logger()

# 使用日志记录器记录日志
logger.debug("这是一个debug信息")
logger.info("这是一个info信息")
logger.warning("这是一个warning信息")
logger.error("这是一个error信息")
logger.critical("这是一个critical信息")


def get_logger():
    return setup_logger()

import { openUrl } from "@tauri-apps/plugin-opener";

const FREE_SUPPORT_URL = "https://my.feishu.cn/wiki/FBvzw4gmeig97JkWYirc8RNv60d";

export function openFreeSupport(): Promise<void> {
  return openUrl(FREE_SUPPORT_URL);
}

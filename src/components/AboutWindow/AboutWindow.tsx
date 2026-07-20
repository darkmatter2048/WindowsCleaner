import { useEffect, useState } from "react";
import {
  makeStyles,
  tokens,
  Text,
  Link,
  Button,
  Spinner,
} from "@fluentui/react-components";
import {
  ArrowSync24Regular,
  CheckmarkCircle24Regular,
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import TitleBar from "../TitleBar/TitleBar";

const useStyles = makeStyles({
  root: {
    display: "grid",
    gridTemplateRows: "auto 1fr",
    height: "100vh",
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: "hidden",
  },
  content: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: "16px",
    padding: "24px",
  },
  appName: {
    fontSize: "28px",
    fontWeight: 800,
    color: tokens.colorNeutralForeground1,
  },
  version: {
    fontSize: "14px",
    color: tokens.colorNeutralForeground3,
    marginTop: "-8px",
  },
  divider: {
    width: "60px",
    height: "2px",
    backgroundColor: tokens.colorBrandForeground1,
    borderRadius: "1px",
    margin: "4px 0",
  },
  links: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "4px",
  },
  link: {
    fontSize: "14px",
  },
  updateSection: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "8px",
    marginTop: "8px",
  },
  updateStatus: {
    fontSize: "13px",
    display: "flex",
    alignItems: "center",
    gap: "4px",
  },
  meta: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "4px",
    marginTop: "16px",
  },
  metaText: {
    fontSize: "12px",
    color: tokens.colorNeutralForeground3,
  },
});

interface VersionInfo {
  version: number;
}

const VERSION_URL = "https://wc.dyblog.online/version.json";

export default function AboutWindow() {
  const styles = useStyles();
  const [version, setVersion] = useState("");
  const [checking, setChecking] = useState(false);
  const [updateMsg, setUpdateMsg] = useState("");
  const [updateOk, setUpdateOk] = useState(true);

  useEffect(() => {
    invoke<string>("fetch_app_version").then(setVersion).catch(() => setVersion("?"));
  }, []);

  const checkUpdate = async () => {
    if (!version) return;
    setChecking(true);
    setUpdateMsg("");
    try {
      const resp = await fetch(VERSION_URL);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const info: VersionInfo = await resp.json();
      // Remote is a plain sequence number; compare with local patch (last segment)
      const currentPatch = parseInt(version.split(".").pop() || "0", 10);
      if (info.version > currentPatch) {
        setUpdateOk(true);
        setUpdateMsg(`发现新版本 v${info.version}，请前往官网下载更新`);
      } else {
        setUpdateOk(true);
        setUpdateMsg("已是最新版本");
      }
    } catch {
      setUpdateOk(false);
      setUpdateMsg("检查更新失败");
    }
    setChecking(false);
  };

  return (
    <div className={styles.root}>
      <TitleBar showIcon={false} title="关于" />

      <div className={styles.content}>
        <Text className={styles.appName}>WindowsCleaner</Text>
        <Text className={styles.version}>v{version || "..."}</Text>

        <div className={styles.divider} />

        <div className={styles.links}>
          <Link className={styles.link} href="https://wc.dyblog.online/" target="_blank">
            官网：wc.dyblog.online
          </Link>
          <Link className={styles.link} href="https://dyblog.online/" target="_blank">
            作者主页：dyblog.online
          </Link>
          <Link className={styles.link} href="https://space.bilibili.com/1847808902" target="_blank">
            作者B站：Mr_Jacek
          </Link>
        </div>

        <div className={styles.updateSection}>
          <Button
            appearance="primary"
            icon={<ArrowSync24Regular />}
            disabled={checking}
            onClick={checkUpdate}
          >
            检查更新
          </Button>
          {checking && <Spinner size="tiny" />}
          {updateMsg && (
            <span className={styles.updateStatus}>
              <CheckmarkCircle24Regular
                style={{
                  fontSize: "14px",
                  color: updateOk
                    ? "var(--colorPaletteGreenForeground1)"
                    : "var(--colorPaletteRedForeground1)",
                }}
              />
              <Text
                style={{
                  color: updateOk
                    ? "var(--colorPaletteGreenForeground1)"
                    : "var(--colorPaletteRedForeground1)",
                }}
              >
                {updateMsg}
              </Text>
            </span>
          )}
        </div>

        <div className={styles.meta}>
          <Text className={styles.metaText}>版权所有 &copy; DaYe</Text>
          <Text className={styles.metaText}>许可证：CC BY-NC-SA 4.0</Text>
        </div>
      </div>
    </div>
  );
}

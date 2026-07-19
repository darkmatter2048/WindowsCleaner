import { useState } from "react";
import { makeStyles, tokens } from "@fluentui/react-components";
import TitleBar from "../TitleBar/TitleBar";
import GeneralCleanPanel from "./GeneralCleanPanel";
import CustomCleanNav, { type CustomCleanSection } from "./CustomCleanNav";
import PlaceholderPanel from "./PlaceholderPanel";

const useStyles = makeStyles({
  root: {
    display: "grid",
    gridTemplateRows: "auto 1fr",
    height: "100vh",
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: "hidden",
  },
  body: {
    display: "grid",
    gridTemplateColumns: "168px 1fr",
    minHeight: 0,
  },
  content: {
    minWidth: 0,
    minHeight: 0,
    overflow: "hidden",
  },
});

export default function CustomCleanWindow() {
  const styles = useStyles();
  const [section, setSection] = useState<CustomCleanSection>("general");

  const renderContent = () => {
    switch (section) {
      case "general":
        return <GeneralCleanPanel />;
      case "scheduled":
        return <PlaceholderPanel titleKey="customClean.nav.scheduled" />;
      case "advanced":
        return <PlaceholderPanel titleKey="customClean.nav.advanced" />;
      case "softwareMove":
        return <PlaceholderPanel titleKey="customClean.nav.softwareMove" />;
      default:
        return <GeneralCleanPanel />;
    }
  };

  return (
    <div className={styles.root}>
      <TitleBar showIcon={false} title="自定义清理" />
      <div className={styles.body}>
        <CustomCleanNav selected={section} onSelect={setSection} />
        <main className={styles.content}>{renderContent()}</main>
      </div>
    </div>
  );
}

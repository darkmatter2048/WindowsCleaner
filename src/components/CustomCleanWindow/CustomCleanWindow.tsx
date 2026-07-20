import { Component, useState } from "react";
import { makeStyles, tokens, Text } from "@fluentui/react-components";
import TitleBar from "../TitleBar/TitleBar";
import GeneralCleanPanel from "./GeneralCleanPanel";
import ScheduledCleanPanel from "./ScheduledCleanPanel";
import AdvancedPanel from "./AdvancedPanel";
import CustomCleanNav, { type CustomCleanSection } from "./CustomCleanNav";
import PlaceholderPanel from "./PlaceholderPanel";
import type { CleanOptionId } from "../../types/clean";
import { DEFAULT_GENERAL_CLEAN_OPTION_IDS } from "../../constants/cleanOptions";

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

// Prevent a single panel crash from blanking the entire window
class PanelErrorBoundary extends Component<
  { children: React.ReactNode },
  { hasError: boolean; errorMsg: string }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, errorMsg: "" };
  }
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, errorMsg: error?.message || String(error) };
  }
  componentDidCatch(error: Error) {
    this.setState({ errorMsg: error?.message || String(error) });
  }
  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: 24 }}>
          <Text style={{ color: "var(--colorNeutralForeground3)", display: "block", marginBottom: 8 }}>
            Something went wrong displaying this panel.
          </Text>
          <Text style={{ fontSize: 12, color: "var(--colorPaletteRedForeground1)", whiteSpace: "pre-wrap" }}>
            {this.state.errorMsg}
          </Text>
        </div>
      );
    }
    return this.props.children;
  }
}

export default function CustomCleanWindow() {
  const styles = useStyles();
  const [section, setSection] = useState<CustomCleanSection>("general");

  // Lifted state so ScheduledCleanPanel can read the user's selected options
  const [selectedOptions, setSelectedOptions] = useState<CleanOptionId[]>(
    DEFAULT_GENERAL_CLEAN_OPTION_IDS,
  );

  const renderContent = () => {
    switch (section) {
      case "general":
        return (
          <GeneralCleanPanel
            selectedOptions={selectedOptions}
            onSelectedOptionsChange={setSelectedOptions}
          />
        );
      case "scheduled":
        return <ScheduledCleanPanel selectedOptions={selectedOptions} />;
      case "advanced":
        return <AdvancedPanel />;
      case "softwareMove":
        return <PlaceholderPanel titleKey="customClean.nav.softwareMove" />;
      default:
        return (
          <GeneralCleanPanel
            selectedOptions={selectedOptions}
            onSelectedOptionsChange={setSelectedOptions}
          />
        );
    }
  };

  return (
    <div className={styles.root}>
      <TitleBar showIcon={false} title="自定义清理" />
      <div className={styles.body}>
        <CustomCleanNav selected={section} onSelect={setSection} />
        <main className={styles.content}>
          <PanelErrorBoundary key={section}>{renderContent()}</PanelErrorBoundary>
        </main>
      </div>
    </div>
  );
}

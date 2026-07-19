import { useState, useCallback } from "react";
import { makeStyles, tokens } from "@fluentui/react-components";
import TitleBar from "./components/TitleBar/TitleBar";
import DiskInfo from "./components/DiskInfo/DiskInfo";
import ActionButtons from "./components/ActionButtons/ActionButtons";
import Footer from "./components/Footer/Footer";
import CustomCleanWindow from "./components/CustomCleanWindow/CustomCleanWindow";

const useStyles = makeStyles({
  root: {
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    height: "100vh",
    backgroundColor: tokens.colorNeutralBackground1,
    overflow: "hidden",
  },
  main: {
    display: "flex",
    flexDirection: "column",
    justifyContent: "center",
    gap: "12px",
    overflow: "hidden",
  },
});

function App() {
  const styles = useStyles();
  const [refreshKey, setRefreshKey] = useState(0);

  const handleCleaned = useCallback(() => {
    setRefreshKey((k) => k + 1);
  }, []);

  if (window.location.hash === "#/custom-clean") {
    return <CustomCleanWindow />;
  }

  return (
    <div className={styles.root}>
      <TitleBar />
      <div className={styles.main}>
        <DiskInfo key={refreshKey} />
        <ActionButtons onCleaned={handleCleaned} />
      </div>
      <Footer />
    </div>
  );
}

export default App;

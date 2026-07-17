import { makeStyles, tokens } from "@fluentui/react-components";
import TitleBar from "./components/TitleBar/TitleBar";
import DiskInfo from "./components/DiskInfo/DiskInfo";
import ActionButtons from "./components/ActionButtons/ActionButtons";
import Footer from "./components/Footer/Footer";

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

  return (
    <div className={styles.root}>
      <TitleBar />
      <div className={styles.main}>
        <DiskInfo />
        <ActionButtons />
      </div>
      <Footer />
    </div>
  );
}

export default App;

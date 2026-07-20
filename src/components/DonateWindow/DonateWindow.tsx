import { makeStyles, tokens } from "@fluentui/react-components";
import TitleBar from "../TitleBar/TitleBar";
import donateQr from "../../assets/donate.png";

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
    alignItems: "center",
    justifyContent: "center",
    padding: "16px",
    overflow: "hidden",
  },
  qr: {
    maxWidth: "100%",
    maxHeight: "100%",
    objectFit: "contain",
    borderRadius: tokens.borderRadiusLarge,
    boxShadow: "0 2px 12px rgba(0,0,0,0.15)",
  },
});

export default function DonateWindow() {
  const styles = useStyles();

  return (
    <div className={styles.root}>
      <TitleBar showIcon={false} title="捐赠" />

      <div className={styles.content}>
        <img src={donateQr} alt="赞赏码" className={styles.qr} />
      </div>
    </div>
  );
}

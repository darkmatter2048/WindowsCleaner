import { Button, makeStyles } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";

const useStyles = makeStyles({
  button: {
    backgroundColor: "#d73f80",
    color: "#ffffff",
    border: "none",
    ":hover": {
      backgroundColor: "#bd2f6d",
      color: "#ffffff",
    },
    ":active": {
      backgroundColor: "#a7235e",
      color: "#ffffff",
    },
  },
});

interface Props {
  onClick: () => void;
  fill?: boolean;
}

export default function FreeSupportButton({ onClick, fill = false }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();

  return (
    <Button
      appearance="primary"
      className={styles.button}
      onClick={onClick}
      style={fill ? { width: "100%" } : undefined}
    >
      {t("support.free")}
    </Button>
  );
}

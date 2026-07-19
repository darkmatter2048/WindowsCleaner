import { makeStyles, tokens, Text } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";

interface Props {
  titleKey: string;
}

const useStyles = makeStyles({
  root: {
    height: "100%",
    display: "flex",
    flexDirection: "column",
    justifyContent: "center",
    alignItems: "center",
    gap: tokens.spacingVerticalS,
    color: tokens.colorNeutralForeground3,
  },
  title: {
    fontSize: "20px",
    fontWeight: 600,
    color: tokens.colorNeutralForeground2,
  },
});

export default function PlaceholderPanel({ titleKey }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();

  return (
    <section className={styles.root}>
      <Text className={styles.title}>{t(titleKey)}</Text>
      <Text>{t("customClean.placeholder")}</Text>
    </section>
  );
}

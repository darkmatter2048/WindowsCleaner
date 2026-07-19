import { makeStyles, tokens, Button } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";

export type CustomCleanSection = "general" | "scheduled" | "advanced" | "softwareMove";

interface Props {
  selected: CustomCleanSection;
  onSelect: (section: CustomCleanSection) => void;
}

const useStyles = makeStyles({
  nav: {
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalXS,
    padding: tokens.spacingHorizontalM,
    backgroundColor: tokens.colorNeutralBackground2,
    borderRight: `1px solid ${tokens.colorNeutralStroke1}`,
    minHeight: 0,
  },
  item: {
    justifyContent: "flex-start",
  },
  active: {
    backgroundColor: tokens.colorBrandBackground2,
  },
});

export default function CustomCleanNav({ selected, onSelect }: Props) {
  const styles = useStyles();
  const { t } = useTranslation();

  const items: Array<{ key: CustomCleanSection; label: string }> = [
    { key: "general", label: t("customClean.nav.general") },
    { key: "scheduled", label: t("customClean.nav.scheduled") },
    { key: "advanced", label: t("customClean.nav.advanced") },
    { key: "softwareMove", label: t("customClean.nav.softwareMove") },
  ];

  return (
    <nav className={styles.nav}>
      {items.map((item) => (
        <Button
          key={item.key}
          appearance={selected === item.key ? "primary" : "subtle"}
          className={`${styles.item} ${selected === item.key ? styles.active : ""}`}
          onClick={() => onSelect(item.key)}
        >
          {item.label}
        </Button>
      ))}
    </nav>
  );
}

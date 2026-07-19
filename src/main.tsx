import ReactDOM from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import i18n from "./i18n";
import { ThemeProvider } from "./contexts/ThemeContext";
import App from "./App";
import "./index.css";

const root = document.getElementById("root") as HTMLElement;

ReactDOM.createRoot(root).render(
  <I18nextProvider i18n={i18n}>
    <ThemeProvider>
      <App />
    </ThemeProvider>
  </I18nextProvider>
);

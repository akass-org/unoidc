import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "virtual:uno.css";
import "@unocss/reset/tailwind.css";
import "./styles/animations.css";
import App from "./App.tsx";
import { initTheme } from "./stores/theme.ts";
import { ToastProvider } from "./components/ui";

function ThemeInitializer() {
  useEffect(() => {
    initTheme();
  }, []);
  return null;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <ToastProvider>
        <ThemeInitializer />
        <App />
      </ToastProvider>
    </BrowserRouter>
  </StrictMode>,
);

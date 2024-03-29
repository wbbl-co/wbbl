import React from "react";
import ReactDOM from "react-dom/client";
import { Theme, ThemePanel } from "@radix-ui/themes";
import App from "./App";

import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme accentColor="lime" grayColor="gray">
      <App />
      <ThemePanel />
    </Theme>
  </React.StrictMode>,
);

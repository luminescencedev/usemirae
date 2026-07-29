import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./app/App";
import "./styles/index.css";

const container = document.getElementById("root");

if (!container) {
  throw new Error("Mirae control UI could not find its #root mount point.");
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

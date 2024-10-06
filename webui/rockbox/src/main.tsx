import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import Providers from "./Providers";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import App from "./App.tsx";
import "./index.css";

const engine = new Styletron();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <StyletronProvider value={engine}>
      <Providers>
        <App />
      </Providers>
    </StyletronProvider>
  </StrictMode>
);

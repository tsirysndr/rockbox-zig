import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import Providers from "./Providers";
import App from "./App.tsx";
import "./index.css";
import GraphQLProvider from "./Providers/GraphQLProvider.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <GraphQLProvider>
      <Providers>
        <App />
      </Providers>
    </GraphQLProvider>
  </StrictMode>
);

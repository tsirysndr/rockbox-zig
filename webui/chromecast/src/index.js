import React from "react";
import ReactDOM from "react-dom/client";
import CastProvider from "./CastProvider";
import CustomReceiver from "./CustomReceiver";
import "./index.css";
import reportWebVitals from "./reportWebVitals";

const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(
  <React.StrictMode>
    <CastProvider>
      <CustomReceiver />
    </CastProvider>
  </React.StrictMode>
);

reportWebVitals();

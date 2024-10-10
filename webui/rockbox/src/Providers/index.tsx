import { FC } from "react";
import ThemeProvider from "./ThemeProvider";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";

const engine = new Styletron();

export type ProvidersProps = {
  children: React.ReactNode;
};

const Providers: FC<ProvidersProps> = ({ children }) => {
  return (
    <StyletronProvider value={engine}>
      <ThemeProvider>{children}</ThemeProvider>
    </StyletronProvider>
  );
};

export default Providers;

import { FC } from "react";
import ThemeProvider from "./ThemeProvider";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import { Provider as JotaiProvider } from "jotai";
import GraphQLProvider from "./GraphQLProvider";

const engine = new Styletron();

export type ProvidersProps = {
  children: React.ReactNode;
};

const Providers: FC<ProvidersProps> = ({ children }) => {
  return (
    <JotaiProvider>
      <GraphQLProvider>
        <StyletronProvider value={engine}>
          <ThemeProvider>{children}</ThemeProvider>
        </StyletronProvider>
      </GraphQLProvider>
    </JotaiProvider>
  );
};

export default Providers;

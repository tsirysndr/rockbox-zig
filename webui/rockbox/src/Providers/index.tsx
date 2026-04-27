import { FC } from "react";
import ThemeProvider from "./ThemeProvider";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import { RecoilRoot } from "recoil";
import GraphQLProvider from "./GraphQLProvider";

const engine = new Styletron();

export type ProvidersProps = {
  children: React.ReactNode;
};

const Providers: FC<ProvidersProps> = ({ children }) => {
  return (
    <RecoilRoot>
      <GraphQLProvider>
        <StyletronProvider value={engine}>
          <ThemeProvider>{children}</ThemeProvider>
        </StyletronProvider>
      </GraphQLProvider>
    </RecoilRoot>
  );
};

export default Providers;

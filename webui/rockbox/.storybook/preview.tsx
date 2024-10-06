import type { Preview } from "@storybook/react";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import React from "react";
import Providers from "../src/Providers";
import "../src/index.css";

const engine = new Styletron();

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
};

export default preview;

export const decorators = [
  (Story) => (
    <StyletronProvider value={engine}>
      <Providers>
        <Story />
      </Providers>
    </StyletronProvider>
  ),
];

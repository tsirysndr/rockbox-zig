import type { Preview } from "@storybook/react";
import React from "react";
import Providers from "../src/Providers";
import "../src/index.css";

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
    <Providers>
      <Story />
    </Providers>
  ),
];

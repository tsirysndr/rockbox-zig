import type { Preview } from "@storybook/react";
import React from "react";
import Providers from "../src/Providers";
import "../src/index.css";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { mocks } from "../src/mocks";
import { MockedProvider } from "@apollo/client/testing";

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

const reactRouterDecorator = (Story) => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/*" element={<Story />} />
      </Routes>
    </MemoryRouter>
  );
};

export const decorators = [
  reactRouterDecorator,
  (Story) => (
    <Providers>
      <MockedProvider mocks={mocks} addTypename={false}>
        <Story />
      </MockedProvider>
    </Providers>
  ),
];

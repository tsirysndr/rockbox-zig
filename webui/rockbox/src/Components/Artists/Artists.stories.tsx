import type { Meta, StoryObj } from "@storybook/react";

import Artists from "./Artists";
import { fn } from "@storybook/test";
import { artists } from "./mocks";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/Artists",
  component: Artists,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
} satisfies Meta<typeof Artists>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    onClickArtist: fn(),
    onFilter: fn(),
    artists,
  },
};

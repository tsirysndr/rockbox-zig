import type { Meta, StoryObj } from "@storybook/react";

import Volume from "./Volume";
import { fn } from "@storybook/test";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/Volume",
  component: Volume,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
  argTypes: {
    volume: {
      control: {
        type: "range",
        min: 0,
        max: 100,
        step: 1,
      },
    },
    onVolumeChange: {
      action: "volume changed",
    },
  },
} satisfies Meta<typeof Volume>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    volume: 70,
    onVolumeChange: fn(),
  },
};

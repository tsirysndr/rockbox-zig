import type { Meta, StoryObj } from "@storybook/react";

import PlayQueue from "./PlayQueue";
import { fn } from "@storybook/test";
import { nextTracks, previousTracks } from "./mocks";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/PlayQueue",
  component: PlayQueue,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
  argTypes: {},
} satisfies Meta<typeof PlayQueue>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    previousTracks,
    nextTracks,
    onPlayTrackAt: fn(),
    onRemoveTrackAt: fn(),
  },
};

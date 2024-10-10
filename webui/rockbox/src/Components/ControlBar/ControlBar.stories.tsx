import type { Meta, StoryObj } from "@storybook/react";

import ControlBar from "./ControlBar";
import { fn } from "@storybook/test";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/ControlBar",
  component: ControlBar,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
  argTypes: {
    onPlay: { action: "onPlay" },
    onPause: { action: "onPause" },
    onNext: { action: "onNext" },
    onPrevious: { action: "onPrevious" },
    onShuffle: { action: "onShuffle" },
    onRepeat: { action: "onRepeat" },
  },
} satisfies Meta<typeof ControlBar>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    onPlay: fn(),
    onPause: fn(),
    onNext: fn(),
    onPrevious: fn(),
    onShuffle: fn(),
    onRepeat: fn(),
  },
};

export const Playing: Story = {
  args: {
    nowPlaying: {
      album: "Forgotten Shapes",
      artist: "Waveshaper",
      title: "Disco on the Baltic Sea",
      cover:
        "https://resources.tidal.com/images/5b33d07f/7d28/417d/8f8f/86291d0b6b34/320x320.jpg",
      duration: 255488.00659179688,
      progress: 123456.789,
      isPlaying: true,
      albumId: "229251493",
    },
    onPlay: fn(),
    onPause: fn(),
    onNext: fn(),
    onPrevious: fn(),
    onShuffle: fn(),
    onRepeat: fn(),
  },
};

import type { Meta, StoryObj } from "@storybook/react";

import ContextMenu from "./ContextMenu";
import { fn } from "@storybook/test";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/Files/ContextMenu",
  component: ContextMenu,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
} satisfies Meta<typeof ContextMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Directory: Story = {
  args: {
    entry: {
      title: "[E]  Live From The Underground [15421743] [2012]",
      isDirectory: true,
    },
    onPlayNext: fn(),
    onCreatePlaylist: fn(),
    onAddTrackToPlaylist: fn(),
    onPlayLast: fn(),
    onAddShuffled: fn(),
    onPlayShuffled: fn(),
    onPlayLastShuffled: fn(),
    recentPlaylists: [],
  },
};

import type { Meta, StoryObj } from "@storybook/react";

import ContextMenu from "./ContextMenu";
import { fn } from "@storybook/test";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/ContextMenu",
  component: ContextMenu,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
} satisfies Meta<typeof ContextMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    track: {
      title: "Drankin N Smokin",
      artist: "Future, Lil Uzi Vert",
      time: "3:34",
      cover:
        "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
    },
    onPlayNext: fn(),
    onCreatePlaylist: fn(),
    onAddTrackToPlaylist: fn(),
    onPlayLast: fn(),
    onAddShuffled: fn(),
    onLike: fn(),
    onUnlike: fn(),
    recentPlaylists: [],
  },
};

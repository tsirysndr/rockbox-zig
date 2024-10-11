import type { Meta, StoryObj } from "@storybook/react";

import AlbumDetails from "./AlbumDetails";
import { fn } from "@storybook/test";
import { tracks } from "./mocks";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/AlbumDetails",
  component: AlbumDetails,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
} satisfies Meta<typeof AlbumDetails>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    onGoBack: fn(),
    onPlayAll: fn(),
    onShuffleAll: fn(),
    onLike: fn(),
    onUnlike: fn(),
    tracks,
    album: {
      id: "1",
      title: "One Cold Night (Live)",
      artist: "Seether",
      year: 2006,
      albumArt:
        "https://resources.tidal.com/images/f6f5f0a6/dc95/4561/9ca6/6ba1e0f6a062/320x320.jpg",
      artistId: "1",
      md5: "md5",
      yearString: "2006",
      tracks: [],
    },
    volumes: [],
  },
};

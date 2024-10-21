import type { Meta, StoryObj } from "@storybook/react";

import ArtistDetails from "./ArtistDetails";
import { tracks, albums } from "./mocks";
import { fn } from "@storybook/test";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories#default-export
const meta = {
  title: "Components/ArtistDetails",
  component: ArtistDetails,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
} satisfies Meta<typeof ArtistDetails>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default: Story = {
  args: {
    name: "Daft Punk",
    tracks,
    albums: albums.map((x) => ({ ...x, cover: x.albumArt })),
    onPlayAll: fn(),
    onShuffleAll: fn(),
    onPlayAlbum: fn(),
    onLikeAlbum: fn(),
    onUnLikeAlbum: fn(),
    onLikeTrack: fn(),
    onUnlikeTrack: fn(),
    onGoBack: fn(),
    onPlayTrack: fn(),
  },
};

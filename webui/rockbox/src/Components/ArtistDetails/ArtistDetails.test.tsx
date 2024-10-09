import { render } from "@testing-library/react";
import { vi } from "vitest";
import ArtistDetails from "./ArtistDetails";
import { albums, tracks } from "./mocks";
import Providers from "../../Providers";

describe("ArtistDetails", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <ArtistDetails
          name="Daft Punk"
          tracks={tracks}
          albums={albums}
          onPlayAll={vi.fn()}
          onShuffleAll={vi.fn()}
          onClickAlbum={vi.fn()}
          onPlayAlbum={vi.fn()}
          onLikeAlbum={vi.fn()}
          onUnLikeAlbum={vi.fn()}
          onLikeTrack={vi.fn()}
          onUnlikeTrack={vi.fn()}
        />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

import { render } from "@testing-library/react";
import { vi } from "vitest";
import ArtistDetails from "./ArtistDetails";
import { albums, tracks } from "./mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";

describe("ArtistDetails", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
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
            onGoBack={vi.fn()}
          />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

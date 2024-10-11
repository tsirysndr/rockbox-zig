import { render } from "@testing-library/react";
import { vi } from "vitest";
import AlbumDetails from "./AlbumDetails";
import { tracks } from "./mocks";
import { MemoryRouter } from "react-router-dom";

describe("AlbumDetails", () => {
  it("should render", () => {
    const onGoBack = vi.fn();
    const onLike = vi.fn();
    const onPlayAll = vi.fn();
    const onShuffleAll = vi.fn();
    const onUnlike = vi.fn();
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <AlbumDetails
          onGoBack={onGoBack}
          onLike={onLike}
          onPlayAll={onPlayAll}
          onShuffleAll={onShuffleAll}
          onUnlike={onUnlike}
          tracks={tracks}
          album={{
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
          }}
          volumes={[]}
        />
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});
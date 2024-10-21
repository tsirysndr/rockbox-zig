import { render } from "@testing-library/react";
import { vi } from "vitest";
import ArtistDetails from "./ArtistDetails";
import { albums, tracks } from "./mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import { mocks } from "../../mocks";

describe("ArtistDetails", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <MockedProvider mocks={mocks}>
          <Providers>
            <ArtistDetails
              name="Daft Punk"
              tracks={tracks}
              albums={albums}
              onPlayAll={vi.fn()}
              onShuffleAll={vi.fn()}
              onPlayAlbum={vi.fn()}
              onLikeAlbum={vi.fn()}
              onUnLikeAlbum={vi.fn()}
              onLikeTrack={vi.fn()}
              onUnlikeTrack={vi.fn()}
              onGoBack={vi.fn()}
              onPlayTrack={vi.fn()}
            />
          </Providers>
        </MockedProvider>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

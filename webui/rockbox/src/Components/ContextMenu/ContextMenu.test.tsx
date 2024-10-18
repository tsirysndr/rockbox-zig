import { render } from "@testing-library/react";
import ContextMenu from "./ContextMenu";
import Providers from "../../Providers";
import { vi } from "vitest";

describe("ContextMenu", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <ContextMenu
          track={{
            title: "Drankin N Smokin",
            artist: "Future, Lil Uzi Vert",
            time: "3:34",
            cover:
              "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
          }}
          onPlayNext={vi.fn()}
          onCreatePlaylist={vi.fn()}
          onAddTrackToPlaylist={vi.fn()}
          onPlayLast={vi.fn()}
          onAddShuffled={vi.fn()}
          recentPlaylists={[]}
        />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

import { render } from "@testing-library/react";
import ContextMenu from "./ContextMenu";
import Providers from "../../../Providers";
import { vi } from "vitest";

describe("ContextMenu", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <ContextMenu
          entry={{
            title: "[E]  Live From The Underground [15421743] [2012]",
            isDirectory: true,
          }}
          onPlayNext={vi.fn()}
          onCreatePlaylist={vi.fn()}
          onAddTrackToPlaylist={vi.fn()}
          onAddShuffled={vi.fn()}
          onPlayShuffled={vi.fn()}
          onPlayLast={vi.fn()}
          onPlayLastShuffled={vi.fn()}
          recentPlaylists={[]}
        />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

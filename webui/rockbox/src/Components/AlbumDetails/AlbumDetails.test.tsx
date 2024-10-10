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
        />
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

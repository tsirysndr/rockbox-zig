import { render } from "@testing-library/react";
import { vi } from "vitest";
import AlbumDetails from "./AlbumDetails";
import { tracks } from "./mocks";

describe("AlbumDetails", () => {
  it("should render", () => {
    const onGoBack = vi.fn();
    const onLike = vi.fn();
    const onPlayAll = vi.fn();
    const onShuffleAll = vi.fn();
    const onUnlike = vi.fn();
    const { container } = render(
      <AlbumDetails
        onGoBack={onGoBack}
        onLike={onLike}
        onPlayAll={onPlayAll}
        onShuffleAll={onShuffleAll}
        onUnlike={onUnlike}
        tracks={tracks}
      />
    );
    expect(container).toMatchSnapshot();
  });
});

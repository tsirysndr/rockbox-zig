import { render } from "@testing-library/react";
import { vi } from "vitest";
import Albums from "./Albums";
import { albums } from "./mocks";
import Providers from "../../Providers";

describe("Albums", () => {
  it("should render", () => {
    const onPlay = vi.fn();
    const onClickAlbum = vi.fn();
    const onFilter = vi.fn();
    const onUnLike = vi.fn();
    const onLike = vi.fn();
    const { container } = render(
      <Providers>
        <Albums
          albums={albums}
          onPlay={onPlay}
          onClickAlbum={onClickAlbum}
          onFilter={onFilter}
          onUnLike={onUnLike}
          onLike={onLike}
        />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

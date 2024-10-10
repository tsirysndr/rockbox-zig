import { render } from "@testing-library/react";
import { vi } from "vitest";
import Albums from "./Albums";
import { albums } from "./mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";

describe("Albums", () => {
  it("should render", () => {
    const onPlay = vi.fn();
    const onFilter = vi.fn();
    const onUnLike = vi.fn();
    const onLike = vi.fn();
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <Providers>
          <Albums
            albums={albums}
            onPlay={onPlay}
            onFilter={onFilter}
            onUnLike={onUnLike}
            onLike={onLike}
          />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

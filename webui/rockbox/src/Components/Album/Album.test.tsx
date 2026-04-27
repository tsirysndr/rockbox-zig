import { render } from "@testing-library/react";
import { vi } from "vitest";
import Album from "./Album";
import { albums } from "../Albums/mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";

describe("Album", () => {
  it("should render", () => {
    const onPlay = vi.fn();
    const onUnLike = vi.fn();
    const onLike = vi.fn();
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        
          <Providers>
            <Album
              album={albums[0]}
              onPlay={onPlay}
              onUnLike={onUnLike}
              onLike={onLike}
            />
          </Providers>
        
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

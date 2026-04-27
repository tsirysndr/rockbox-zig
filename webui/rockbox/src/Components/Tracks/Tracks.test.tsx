import { render } from "@testing-library/react";
import Tracks from "./Tracks";
import Providers from "../../Providers";
import { tracks } from "./mocks";
import { MemoryRouter } from "react-router-dom";
import { vi } from "vitest";

describe("Tracks", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        
          <Providers>
            <Tracks tracks={tracks} onPlayTrack={vi.fn()} />
          </Providers>
        
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

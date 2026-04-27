import { render } from "@testing-library/react";
import { vi } from "vitest";
import PlayQueue from "./PlayQueue";
import Providers from "../../../Providers";
import { MemoryRouter } from "react-router-dom";
import { nextTracks, previousTracks } from "./mocks";

describe("PlayQueue", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <MemoryRouter initialEntries={["/"]}>
          
            <PlayQueue
              previousTracks={previousTracks}
              nextTracks={nextTracks}
              currentTrack={undefined}
              onPlayTrackAt={vi.fn()}
              onRemoveTrackAt={vi.fn()}
            />
          
        </MemoryRouter>
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

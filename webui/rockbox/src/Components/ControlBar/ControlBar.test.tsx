import { render } from "@testing-library/react";
import { vi } from "vitest";
import ControlBar from "./ControlBar";
import Providers from "../../Providers";
import { MockedProvider } from "@apollo/client/testing";
import { mocks } from "../../mocks";
import { MemoryRouter } from "react-router-dom";

describe("ControlBar", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <MemoryRouter initialEntries={["/"]}>
          <MockedProvider mocks={mocks}>
            <ControlBar
              nowPlaying={{
                album: "Forgotten Shapes",
                artist: "Waveshaper",
                title: "Disco on the Baltic Sea",
                cover:
                  "https://resources.tidal.com/images/5b33d07f/7d28/417d/8f8f/86291d0b6b34/320x320.jpg",
                duration: 255488.00659179688,
                progress: 123456.789,
                isPlaying: true,
                albumId: "229251493",
              }}
              onPlay={vi.fn()}
              onPause={vi.fn()}
              onNext={vi.fn()}
              onPrevious={vi.fn()}
              onShuffle={vi.fn()}
              onRepeat={vi.fn()}
              onLike={vi.fn()}
              onUnlike={vi.fn()}
              onSeek={vi.fn()}
            />
          </MockedProvider>
        </MemoryRouter>
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

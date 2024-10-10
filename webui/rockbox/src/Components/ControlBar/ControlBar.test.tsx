import { render } from "@testing-library/react";
import { vi } from "vitest";
import ControlBar from "./ControlBar";
import Providers from "../../Providers";

describe("ControlBar", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
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
        />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

import { render } from "@testing-library/react";
import { vi } from "vitest";
import Volume from "./Volume";
import Providers from "../../../../Providers";

describe("Volume", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Volume volume={70} onVolumeChange={vi.fn()} />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

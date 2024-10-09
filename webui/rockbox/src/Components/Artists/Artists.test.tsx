import { render } from "@testing-library/react";
import { vi } from "vitest";
import Artists from "./Artists";
import { artists } from "./mocks";
import Providers from "../../Providers";

describe("Artists", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Artists artists={artists} onClickArtist={vi.fn()} onFilter={vi.fn()} />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

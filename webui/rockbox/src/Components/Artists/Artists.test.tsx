import { render } from "@testing-library/react";
import { vi } from "vitest";
import Artists from "./Artists";
import { artists } from "./mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";

describe("Artists", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <Providers>
          <Artists
            artists={artists}
            onClickArtist={vi.fn()}
            onFilter={vi.fn()}
          />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

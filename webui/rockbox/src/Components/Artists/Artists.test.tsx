import { render } from "@testing-library/react";
import { vi } from "vitest";
import Artists from "./Artists";
import { artists } from "./mocks";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import { mocks } from "../../mocks";

describe("Artists", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <MockedProvider mocks={mocks}>
          <Providers>
            <Artists
              artists={artists}
              onClickArtist={vi.fn()}
              onFilter={vi.fn()}
            />
          </Providers>
        </MockedProvider>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

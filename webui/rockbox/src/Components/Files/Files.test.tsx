import { render } from "@testing-library/react";
import Files from "./Files";
import Providers from "../../Providers";
import { files } from "./mocks";
import { MemoryRouter } from "react-router-dom";
import { vi } from "vitest";
import { MockedProvider } from "@apollo/client/testing";
import { mocks } from "../../mocks";

describe("Files", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <MockedProvider mocks={mocks}>
          <Providers>
            <Files
              files={files}
              canGoBack={true}
              onGoBack={vi.fn()}
              onPlayDirectory={vi.fn()}
              onPlayTrack={vi.fn()}
            />
          </Providers>
        </MockedProvider>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

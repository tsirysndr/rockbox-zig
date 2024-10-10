import { render } from "@testing-library/react";
import Tracks from "./Tracks";
import Providers from "../../Providers";
import { tracks } from "./mocks";
import { MemoryRouter } from "react-router-dom";

describe("Tracks", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <Providers>
          <Tracks tracks={tracks} />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

import { render } from "@testing-library/react";
import Files from "./Files";
import Providers from "../../Providers";
import { files } from "./mocks";
import { MemoryRouter } from "react-router-dom";

describe("Files", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <Providers>
          <Files files={files} />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

import { render } from "@testing-library/react";
import Sidebar from "./Sidebar";
import Providers from "../../Providers";
import { MemoryRouter } from "react-router-dom";

describe("Sidebar", () => {
  it("should render", () => {
    const { container } = render(
      <MemoryRouter initialEntries={["/"]}>
        <Providers>
          <Sidebar active="albums" />
        </Providers>
      </MemoryRouter>
    );
    expect(container).toMatchSnapshot();
  });
});

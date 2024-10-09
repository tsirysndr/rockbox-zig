import { render } from "@testing-library/react";
import Sidebar from "./Sidebar";
import Providers from "../../Providers";

describe("Sidebar", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Sidebar active="albums" />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

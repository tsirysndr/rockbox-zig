import { render } from "@testing-library/react";
import { vi } from "vitest";
import Filter from "./Filter";
import Providers from "../../Providers";

describe("Filter", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Filter placeholder="Search Songs" onChange={vi.fn()} />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

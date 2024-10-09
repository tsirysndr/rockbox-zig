import { render } from "@testing-library/react";
import { vi } from "vitest";
import Button from "./Button";
import Providers from "../../Providers";

describe("Button", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Button onClick={vi.fn()}>Click me</Button>
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

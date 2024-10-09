import { render } from "@testing-library/react";
import Files from "./Files";
import Providers from "../../Providers";
import { files } from "./mocks";

describe("Files", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Files files={files} />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

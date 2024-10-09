import { render } from "@testing-library/react";
import Tracks from "./Tracks";
import Providers from "../../Providers";
import { tracks } from "./mocks";

describe("Tracks", () => {
  it("should render", () => {
    const { container } = render(
      <Providers>
        <Tracks tracks={tracks} />
      </Providers>
    );
    expect(container).toMatchSnapshot();
  });
});

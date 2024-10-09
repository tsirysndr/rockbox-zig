import { render } from "@testing-library/react";
import AlbumDetails from "./AlbumDetails";
import { tracks } from "./mocks";

describe("AlbumDetails", () => {
  it("should render", () => {
    const { container } = render(
      <AlbumDetails
        onGoBack={() => {}}
        onLike={() => {}}
        onPlayAll={() => {}}
        onShuffleAll={() => {}}
        onUnlike={() => {}}
        tracks={tracks}
      />
    );
    expect(container).toMatchSnapshot();
  });
});

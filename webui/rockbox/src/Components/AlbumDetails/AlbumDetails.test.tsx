import { render } from "@testing-library/react";
import AlbumDetails from "./AlbumDetails";

describe("AlbumDetails", () => {
  it("should render", () => {
    const screen = render(<AlbumDetails />);
    screen.debug();
  });
});

import { FC } from "react";
import { Link } from "react-router-dom";
import { Cell, Grid } from "baseui/layout-grid";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import type { GenreSummary } from "../../Hooks/useGenres";

export function colorForSeed(seed: string): string {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
  }
  const hue = hash % 360;
  return `hsl(${hue} 65% 38%)`;
}

export type GenresProps = {
  genres: GenreSummary[];
  loading?: boolean;
};

const Genres: FC<GenresProps> = ({ genres, loading }) => {
  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="genres" />
      <MainView>
        <ControlBar />
        <div className="h-[calc(100vh-60px)] overflow-y-auto">
          <div className="text-2xl font-[RockfordSansMedium] max-w-[96%] mx-auto mb-5 px-5">
            Genres
          </div>
          {!loading && genres.length === 0 ? (
            <div
              style={{
                padding: 20,
                color: "#999",
                fontSize: 14,
                paddingLeft: 30,
              }}
            >
              No genres found yet.
            </div>
          ) : null}
          {genres.length > 0 && (
            <div style={{ marginBottom: 100 }}>
              <Grid
                gridColumns={[2, 3, 4]}
                gridMargins={[20, 20, 20]}
                gridGutters={[16, 16, 16]}
              >
                {genres.map((g) => (
                  <Cell key={g.id}>
                    <Link
                      to={`/genres/${g.id}`}
                      style={{ backgroundColor: colorForSeed(g.id) }}
                      className="relative block rounded-[8px] overflow-hidden h-[110px] mb-6 no-underline text-white cursor-pointer hover:brightness-110"
                    >
                      <div className="absolute top-3 left-[14px] right-[14px] font-[RockfordSansMedium] text-[18px] font-semibold z-[1]">
                        {g.name}
                      </div>
                      <div className="absolute bottom-[10px] left-[14px] text-[11px] opacity-85 z-[1]">
                        {g.trackCount} {g.trackCount === 1 ? "track" : "tracks"}
                      </div>
                      <div className="absolute right-[-16px] bottom-[-16px] font-[RockfordSansMedium] text-[36px] font-bold opacity-[0.18] rotate-[-12deg] z-0">
                        {g.name}
                      </div>
                    </Link>
                  </Cell>
                ))}
              </Grid>
            </div>
          )}
        </div>
      </MainView>
    </div>
  );
};

export default Genres;

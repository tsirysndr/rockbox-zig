import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  Container,
  GenreCard,
  GenreCount,
  GenreDecoration,
  GenreLabel,
  Scrollable,
  Title,
  colorForSeed,
} from "./styles";
import type { GenreSummary } from "../../Hooks/useGenres";

export type GenresProps = {
  genres: GenreSummary[];
  loading?: boolean;
};

const Genres: FC<GenresProps> = ({ genres, loading }) => {
  return (
    <Container>
      <Sidebar active="genres" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <Title>Genres</Title>
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
                    <GenreCard to={`/genres/${g.id}`} bg={colorForSeed(g.id)}>
                      <GenreLabel>{g.name}</GenreLabel>
                      <GenreCount>
                        {g.trackCount} {g.trackCount === 1 ? "track" : "tracks"}
                      </GenreCount>
                      <GenreDecoration>{g.name}</GenreDecoration>
                    </GenreCard>
                  </Cell>
                ))}
              </Grid>
            </div>
          )}
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Genres;

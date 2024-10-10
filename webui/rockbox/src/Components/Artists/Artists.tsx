/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  ArtistCover,
  ArtistName,
  Container,
  MainView,
  NoArtistCover,
  Scrollable,
  Title,
} from "./styles";
import Artist from "../Icons/Artist";
import { Link } from "react-router-dom";

export type ArtistsProps = {
  artists: any[];
  onClickArtist: (artist: any) => void;
  onFilter: (filter: string) => void;
};

const Artists: FC<ArtistsProps> = (props) => {
  const { onClickArtist, artists } = props;
  return (
    <Container>
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <Title>Artists</Title>
          <div style={{ marginBottom: 100 }}>
            <Grid
              gridColumns={[2, 3, 4]}
              gridMargins={[18, 18, 18]}
              gridGutters={[10, 10, 10]}
            >
              {artists.map((item) => (
                <Cell key={item.id} align={"center"}>
                  <Link
                    to={`/artists/${item.id}`}
                    style={{ textDecoration: "none" }}
                  >
                    {item.cover && (
                      <ArtistCover
                        src={item.cover}
                        onClick={() => onClickArtist(item)}
                      />
                    )}
                    {!item.cover && (
                      <NoArtistCover onClick={() => onClickArtist(item)}>
                        <Artist width={75} height={75} color="#a4a3a3" />
                      </NoArtistCover>
                    )}
                    <ArtistName>{item.name}</ArtistName>
                  </Link>
                </Cell>
              ))}
            </Grid>
          </div>
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Artists;

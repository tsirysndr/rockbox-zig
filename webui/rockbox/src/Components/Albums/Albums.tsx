/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import AlbumArt from "../../Assets/albumart.png";
import {
  AlbumCover,
  AlbumTitle,
  Artist,
  Container,
  FilterContainer,
  MainView,
  Scrollable,
  Title,
  Year,
} from "./styles";
import Filter from "../Filter";

export type AlbumsProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
  onFilter: (filter: string) => void;
};

const Albums: FC<AlbumsProps> = (props) => {
  const { albums, onClickAlbum } = props;
  return (
    <Container>
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <Title>Albums</Title>
          <FilterContainer>
            <Filter placeholder="Search albums" onChange={() => {}} />
          </FilterContainer>
          <div style={{ marginBottom: 100 }}>
            <Grid gridColumns={[2, 4, 5]} gridMargins={[8, 10, 12]}>
              {albums.map((item) => (
                <Cell key={item.id}>
                  <AlbumCover
                    src={item.cover ? item.cover : AlbumArt}
                    onClick={() => onClickAlbum(item)}
                    effect="opacity"
                  />
                  <AlbumTitle onClick={() => onClickAlbum(item)}>
                    {item.title}
                  </AlbumTitle>
                  <Artist>{item.artist}</Artist>
                  <Year>{item.year}</Year>
                </Cell>
              ))}
            </Grid>
          </div>
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Albums;

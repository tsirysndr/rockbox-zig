import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
// import { useWindowVirtualizer } from "@tanstack/react-virtual";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import AlbumIcon from "../Icons/AlbumCover";
import {
  AlbumCover,
  AlbumTitle,
  Artist,
  Container,
  MainView,
  NoAlbumCover,
  Scrollable,
  Title,
  Year,
} from "./styles";

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
          <div style={{ marginBottom: 100 }}>
            <Grid gridColumns={[2, 4, 5]} gridMargins={[8, 16, 18]}>
              {albums.map((item) => (
                <Cell key={item.id}>
                  {item.cover && (
                    <AlbumCover
                      src={item.cover}
                      onClick={() => onClickAlbum(item)}
                    />
                  )}
                  {!item.cover && (
                    <NoAlbumCover onClick={() => onClickAlbum(item)}>
                      <AlbumIcon />
                    </NoAlbumCover>
                  )}
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

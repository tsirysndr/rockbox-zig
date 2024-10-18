/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import AlbumArt from "../../Assets/albumart.svg";
import {
  AlbumCover,
  AlbumFooterMenu,
  AlbumTitle,
  Artist,
  Container,
  FilterContainer,
  FloatingButton,
  Hover,
  Scrollable,
  Title,
  Year,
  Link,
  NoAlbumCover,
} from "./styles";
import Filter from "../Filter";
import Play from "../Icons/Play";
import HeartOutline from "../Icons/HeartOutline";
import ContextMenu from "./ContextMenu";

export type AlbumsProps = {
  albums: any[];
  onFilter: (filter: string) => void;
  onPlay: (album: any) => void;
  onLike: (album: any) => void;
  onUnLike: (album: any) => void;
};

const Albums: FC<AlbumsProps> = (props) => {
  const { albums } = props;
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
            <Grid
              gridColumns={[2, 4, 5]}
              gridMargins={[20, 20, 20]}
              gridGutters={[25, 25, 25]}
            >
              {albums.map((item) => (
                <Cell key={item.id}>
                  <div style={{ position: "relative", width: "100%" }}>
                    <Hover>
                      <AlbumFooterMenu>
                        <div
                          style={{
                            backgroundColor: "#ffffffda",
                            height: 40,
                            width: 40,
                            borderRadius: 20,
                            display: "flex",
                            justifyContent: "center",
                            alignItems: "center",
                          }}
                          onClick={() => props.onPlay(item)}
                        >
                          <Play small color="#000" />
                        </div>
                        <ContextMenu
                          album={item}
                          onPlayNext={() => {}}
                          onCreatePlaylist={() => {}}
                          onAddTrackToPlaylist={() => {}}
                          onPlayLast={() => {}}
                          recentPlaylists={[]}
                        />
                        <FloatingButton onClick={() => props.onLike(item)}>
                          <HeartOutline color="#fff" size={20} />
                        </FloatingButton>
                      </AlbumFooterMenu>
                    </Hover>
                    <Link to={`/albums/${item.id}`}>
                      {item.cover && (
                        <AlbumCover src={item.cover} effect="opacity" />
                      )}
                      {!item.cover && <NoAlbumCover src={AlbumArt} />}
                      <AlbumTitle>{item.title}</AlbumTitle>
                    </Link>
                    <Artist to={`/artists/${item.artistId}`}>
                      {item.artist}
                    </Artist>
                    <Year>{item.year}</Year>
                  </div>
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

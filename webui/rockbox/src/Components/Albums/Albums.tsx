/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import AlbumArt from "../../Assets/albumart.png";
import {
  AlbumCover,
  AlbumFooterMenu,
  AlbumTitle,
  Artist,
  Container,
  FilterContainer,
  FloatingButton,
  Hover,
  MainView,
  Scrollable,
  Title,
  Year,
} from "./styles";
import Filter from "../Filter";
import Play from "../Icons/Play";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import HeartOutline from "../Icons/HeartOutline";

export type AlbumsProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
  onFilter: (filter: string) => void;
  onPlay: (album: any) => void;
  onLike: (album: any) => void;
  onUnLike: (album: any) => void;
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
                        <FloatingButton>
                          <EllipsisHorizontal size={24} color="#fff" />
                        </FloatingButton>
                        <FloatingButton onClick={() => props.onLike(item)}>
                          <HeartOutline color="#fff" />
                        </FloatingButton>
                      </AlbumFooterMenu>
                    </Hover>
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

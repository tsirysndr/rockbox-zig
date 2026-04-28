/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Container, FilterContainer, Scrollable, Title } from "./styles";
import Filter from "../Filter";
import Album from "../Album";
import AlbumCardSkeleton from "../Skeletons/AlbumCardSkeleton";

export type AlbumsProps = {
  albums: any[];
  onFilter: (filter: string) => void;
  onLike: (album: any) => void;
  onUnLike: (album: any) => void;
  keyword?: string;
  loading?: boolean;
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
          {props.loading && (
            <div style={{ marginBottom: 100 }}>
              <Grid
                gridColumns={[2, 4, 5]}
                gridMargins={[20, 20, 20]}
                gridGutters={[25, 25, 25]}
              >
                {Array.from({ length: 10 }).map((_, i) => (
                  <Cell key={i}>
                    <AlbumCardSkeleton />
                  </Cell>
                ))}
              </Grid>
            </div>
          )}
          {(props.albums.length > 0 || props.keyword) && !props.loading && (
            <>
              <FilterContainer>
                <Filter placeholder="Search albums" />
              </FilterContainer>
              <div style={{ marginBottom: 100 }}>
                <Grid
                  gridColumns={[2, 4, 5]}
                  gridMargins={[20, 20, 20]}
                  gridGutters={[25, 25, 25]}
                >
                  {albums.map((item) => (
                    <Cell key={item.id}>
                      <Album album={item} />
                    </Cell>
                  ))}
                </Grid>
              </div>
            </>
          )}
        </Scrollable>
      </MainView>
    </Container>
  );
};

export default Albums;

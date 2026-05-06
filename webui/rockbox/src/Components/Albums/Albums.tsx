/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
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
    <div className="flex flex-row w-full h-full">
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
        <div className="h-[calc(100vh-60px)] overflow-y-auto">
          <div className="text-2xl font-[RockfordSansMedium] max-w-[96%] mx-auto mb-5 px-5">Albums</div>
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
              <div className="mt-[30px] mb-10 px-5">
                <Filter placeholder="Search albums" />
              </div>
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
        </div>
      </MainView>
    </div>
  );
};

export default Albums;

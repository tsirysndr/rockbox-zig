/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar";
import MainView from "../MainView";
import ControlBar from "../ControlBar";
import Artist from "../Icons/Artist";
import { Link } from "react-router-dom";
import Filter from "../Filter";
import ArtistCardSkeleton from "../Skeletons/ArtistCardSkeleton";

export type ArtistsProps = {
  artists: any[];
  onClickArtist: (artist: any) => void;
  onFilter: (filter: string) => void;
  keyword?: string;
  loading?: boolean;
};

const Artists: FC<ArtistsProps> = (props) => {
  const { onClickArtist, artists } = props;
  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
        <div className="h-[var(--content-area-height)] overflow-y-auto">
          <div className="text-2xl font-[RockfordSansMedium] mx-auto mb-5 px-5">
            Artists
          </div>
          {props.loading && (
            <div style={{ marginBottom: 100 }}>
              <Grid
                gridColumns={[2, 3, 4]}
                gridMargins={[18, 18, 18]}
                gridGutters={[10, 10, 10]}
              >
                {Array.from({ length: 8 }).map((_, i) => (
                  <Cell key={i} align="center">
                    <ArtistCardSkeleton />
                  </Cell>
                ))}
              </Grid>
            </div>
          )}
          {(props.artists.length > 0 || props.keyword) && !props.loading && (
            <>
              <div className="mt-[30px] mb-10 ml-5">
                <Filter placeholder="Search artists" />
              </div>
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
                          <img
                            className="w-[194px] h-[194px] rounded-[97px] cursor-pointer"
                            src={item.cover}
                            onClick={() => onClickArtist(item)}
                          />
                        )}
                        {!item.cover && (
                          <div
                            className="w-[194px] h-[194px] rounded-[97px] cursor-pointer flex justify-center items-center bg-[var(--theme-cover)]"
                            onClick={() => onClickArtist(item)}
                          >
                            <Artist width={75} height={75} color="#a4a3a3" />
                          </div>
                        )}
                        <div className="text-sm text-ellipsis overflow-hidden whitespace-nowrap cursor-pointer mt-5 mb-[18px] text-center w-[194px] text-[var(--theme-text)]">
                          {item.name}
                        </div>
                      </Link>
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

export default Artists;

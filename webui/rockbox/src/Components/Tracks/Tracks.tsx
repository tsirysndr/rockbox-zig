/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC, useRef } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import MainView from "../MainView";
import { Link as RouterLink } from "react-router-dom";
import { LazyLoadImage } from "react-lazy-load-image-component";
import { Track } from "../../Types/track";
import Table from "../VirtualizedTable";
import Filter from "../Filter";
import TrackIcon from "../Icons/Track";
import { Play } from "@styled-icons/ionicons-sharp";
import "./styles.css";
import ContextMenu from "../ContextMenu";
import TrackListSkeleton from "../Skeletons/TrackListSkeleton";

const columnHelper = createColumnHelper<Track>();

export type TracksProps = {
  tracks: Track[];
  onPlayTrack: (index: number) => void;
  keyword?: string;
  loading?: boolean;
};

const Tracks: FC<TracksProps> = (props) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const columns = [
    columnHelper.accessor("trackNumber", {
      header: "#",
      size: 20,
      cell: (info) => (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginLeft: 5,
            marginRight: 5,
            marginTop: -6,
          }}
        >
          {info.getValue()}
        </div>
      ),
    }),
    columnHelper.accessor("albumArt", {
      header: "Title",
      size: 54,
      cell: (info) => (
        <>
          {info.getValue() && (
            <div className="album-cover-container songs">
              <LazyLoadImage
                className="h-12 w-12"
                src={info.getValue()!}
                alt="album art"
                effect="blur"
              />
              <div
                onClick={() => props.onPlayTrack(info.row.index)}
                className="floating-play"
              >
                <Play size={16} color={info.getValue() ? "#fff" : "var(--theme-text)"} />
              </div>
            </div>
          )}
          {!info.getValue() && (
            <div className="album-cover-container songs">
              <div className="h-12 w-12 rounded-[4px] cursor-pointer bg-[var(--theme-cover)] flex justify-center items-center">
                <TrackIcon width={28} height={28} color="#a4a3a3" />
              </div>
              <div
                onClick={() => props.onPlayTrack(info.row.index)}
                className="floating-play"
              >
                <Play size={16} color={info.getValue() ? "#fff" : "var(--theme-text)"} />
              </div>
            </div>
          )}
        </>
      ),
    }),
    columnHelper.accessor("title", {
      header: "",
      cell: (info) => (
        <div
          style={{
            minWidth: 150,
            maxWidth: "calc((100vw - 240px - 230px) / 3)",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
            color: "var(--theme-text)",
          }}
        >
          {info.getValue()}
        </div>
      ),
    }),
    columnHelper.accessor("artist", {
      header: "Artist",
      cell: (info) => (
        <div
          style={{
            minWidth: 150,
            maxWidth: 170,
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
            color: "var(--theme-text)",
          }}
        >
          <RouterLink
            className="text-[var(--theme-text)] no-underline font-[RockfordSansRegular] hover:underline"
            to={`/artists/${info.row.original.artistId}`}
          >
            {info.getValue()}
          </RouterLink>
        </div>
      ),
    }),
    columnHelper.accessor("album", {
      header: "Album",
      cell: (info) => (
        <div
          style={{
            minWidth: 150,
            maxWidth: "calc((100vw - 240px - 230px) / 3)",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
            color: "var(--theme-text)",
          }}
        >
          <RouterLink
            className="text-[var(--theme-text)] no-underline font-[RockfordSansRegular] hover:underline"
            to={`/albums/${info.row.original.albumId}`}
          >
            {info.getValue()}
          </RouterLink>
        </div>
      ),
    }),
    columnHelper.accessor("time", {
      header: "Time",
      size: 50,
      cell: (info) => info.getValue(),
    }),
    columnHelper.accessor("id", {
      header: "",
      size: 100,
      cell: (info) => (
        <div
          className="flex flex-row items-center"
          style={{ justifyContent: "flex-end", alignItems: "center" }}
        >
          <ContextMenu
            track={{
              id: info.row.original.id,
              title: info.row.original.title,
              artist: info.row.original.artist,
              time: info.row.original.time,
              cover: info.row.original.albumArt,
              path: info.row.original.path,
            }}
          />
        </div>
      ),
    }),
  ];
  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="songs" />
      <MainView>
        <ControlBar />
        <div className="overflow-y-auto h-[var(--content-area-height)] px-5 relative" ref={containerRef}>
          <div className="text-2xl font-[RockfordSansMedium] mb-5">Songs</div>
          {props.loading && (
            <div style={{ marginBottom: 60 }}>
              <TrackListSkeleton />
            </div>
          )}
          {(props.tracks.length > 0 || props.keyword) && !props.loading && (
            <>
              <div className="mt-[30px] mb-10">
                <Filter placeholder="Search song" />
              </div>
              <div style={{ marginBottom: 60 }}>
                {props.tracks.length > 0 && (
                  <Table
                    columns={columns as any}
                    tracks={props.tracks}
                    containerRef={containerRef}
                  />
                )}
              </div>
            </>
          )}
        </div>
      </MainView>
    </div>
  );
};

export default Tracks;

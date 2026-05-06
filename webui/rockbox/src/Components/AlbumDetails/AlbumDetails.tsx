/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import { Link } from "react-router-dom";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import MainView from "../MainView/MainView";
import Button from "../Button";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import Table from "../Table";
import { Track } from "../../Types/track";
import { Album } from "../../Hooks/GraphQL";
import AlbumArt from "../../Assets/albumart.svg";
import ContextMenu from "../ContextMenu";
import _ from "lodash";
import DetailHeaderSkeleton from "../Skeletons/DetailHeaderSkeleton";
import TrackListSkeleton from "../Skeletons/TrackListSkeleton";

const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

function formatReleaseDate(s: string): string {
  const parts = s.split("-");
  if (parts.length === 3) {
    const y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const d = parseInt(parts[2], 10);
    if (!isNaN(y) && !isNaN(m) && !isNaN(d) && m >= 1 && m <= 12) {
      return `${d} ${MONTHS[m - 1]} ${y}`;
    }
  }
  return s;
}

const columnHelper = createColumnHelper<Track>();

export type AlbumDetailsProps = {
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onGoBack: () => void;
  tracks: Track[];
  album?: Album | null;
  volumes: Track[][];
  enableBlur?: boolean;
  loading?: boolean;
  onPlayTrack: (position: number, disc: number) => void;
};

const AlbumDetails: FC<AlbumDetailsProps> = (props) => {
  const columns = [
    columnHelper.accessor("trackNumber", {
      header: "#",
      size: 20,
      cell: (info) => (
        <div style={{ position: "relative" }}>
          <div className="tracknumber">{info.getValue()}</div>
          <div
            className="floating-play"
            onClick={() =>
              props.onPlayTrack(
                info.row.index,
                _.get(info, "row.original.discnum", 1)
              )
            }
          >
            <Play color="var(--theme-icon)" small />
          </div>
        </div>
      ),
    }),
    columnHelper.accessor("title", {
      header: "Title",
      cell: (info) => (
        <div
          style={{
            minWidth: 150,
            width: "calc(100% - 20px)",
            maxWidth: "300px",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
          }}
          className="text-text"
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
            width: "calc(100% - 20px)",
            maxWidth: "calc(100vw - 800px)",
            fontSize: 14,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            cursor: "pointer",
          }}
          className="text-text"
        >
          <Link
            className="text-text no-underline font-[RockfordSansRegular] hover:underline"
            to={`/artists/${info.row.original.artistId}`}
          >
            {info.getValue()}
          </Link>
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
        <div className="flex flex-row items-center" style={{ justifyContent: "flex-end", alignItems: "center" }}>
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
      <Sidebar active="albums" />
      <MainView
        cover={props.enableBlur ? (props.album?.albumArt as any) : undefined}
      >
        <ControlBar />
        <div className="pl-[30px] pr-[30px] overflow-y-auto h-[calc(100vh-60px)]">
          <button
            className="border-0 cursor-pointer flex items-center justify-center h-[30px] w-[30px] rounded-[15px] bg-back-button mt-[26px] mb-[46px] absolute z-[1]"
            onClick={() => props.onGoBack()}
          >
            <div style={{ marginTop: 2 }}>
              <ArrowBack color="var(--theme-icon)" />
            </div>
          </button>
          {props.loading && (
            <div style={{ marginTop: 60, marginBottom: 100 }}>
              <DetailHeaderSkeleton />
              <TrackListSkeleton />
            </div>
          )}
          {!props.loading && <div style={{ marginBottom: 100 }}>
            <div className="flex flex-row items-center mb-5 mt-[90px]">
              <img
                className="h-[240px] w-[240px] rounded-[3px]"
                src={props.album?.albumArt || AlbumArt}
              />
              <div className="flex flex-col ml-[26px] h-[240px] justify-center">
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "center",
                    height: "calc(240px - 12px)",
                  }}
                >
                  <div className="text-[32px] font-[RockfordSansBold] text-text">
                    {props.album?.title}
                  </div>
                  <Link
                    className="text-text no-underline font-[RockfordSansMedium] text-sm mt-2 hover:underline"
                    to={`/artists/${props.album?.artistId}`}
                  >
                    {props.album?.artist}
                  </Link>
                  <div className="mt-[25px] font-normal text-sm text-secondary-text">
                    {props.tracks.length || props.volumes.flat().length} TRACKS
                  </div>
                  <div className="mt-[15px] font-normal text-sm mb-[10px] text-secondary-text">
                    {props.album?.year}
                  </div>
                </div>
                <div className="flex flex-row items-center">
                  <Button onClick={() => props.onPlayAll()} kind="primary">
                    <div className="flex flex-row items-center">
                      <Play small color="#fff" />
                      <div style={{ marginLeft: 7 }}>Play</div>
                    </div>
                  </Button>
                  <div className="w-5" />
                  <Button onClick={() => props.onShuffleAll()} kind="secondary">
                    <div className="flex flex-row items-center">
                      <Shuffle color="#6F00FF" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </div>
                  </Button>
                </div>
              </div>
            </div>
            {props.volumes.length === 0 && props.tracks.length > 0 && (
              <Table columns={columns as any} tracks={props.tracks} />
            )}
            {props.volumes.length > 0 && (
              <div>
                {props.volumes.map((volume, index) => (
                  <div key={index}>
                    <div
                      style={{ fontSize: 18, fontWeight: 500, marginTop: 20 }}
                    >
                      Volume {index + 1}
                    </div>
                    <Table columns={columns as any} tracks={volume} />
                  </div>
                ))}
              </div>
            )}
            {(props.album?.yearString || props.album?.copyrightMessage) && (
              <div className="flex flex-col gap-1 py-6 pb-8">
                {props.album?.yearString && (
                  <div className="text-xs text-secondary-text">
                    {formatReleaseDate(props.album.yearString)}
                  </div>
                )}
                {props.album?.copyrightMessage && (
                  <div className="text-xs text-secondary-text">
                    {props.album.copyrightMessage}
                  </div>
                )}
              </div>
            )}
          </div>}
        </div>
      </MainView>
    </div>
  );
};

export default AlbumDetails;

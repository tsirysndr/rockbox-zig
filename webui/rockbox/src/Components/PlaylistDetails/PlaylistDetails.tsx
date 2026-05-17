/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import MainView from "../MainView/MainView";
import { Link as RouterLink } from "react-router-dom";
import Button from "../Button";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import Table from "../Table";
import { Track } from "../../Types/track";
import ContextMenu from "../ContextMenu";
import { Music } from "@styled-icons/boxicons-regular";
import DetailHeaderSkeleton from "../Skeletons/DetailHeaderSkeleton";
import TrackListSkeleton from "../Skeletons/TrackListSkeleton";

const columnHelper = createColumnHelper<Track>();

export type PlaylistDetailsProps = {
  playlist?: any;
  tracks: Track[];
  isSmart?: boolean;
  loading?: boolean;
  onGoBack: () => void;
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onPlayTrack: (position: number) => void;
  onRemoveTrack?: (trackId: string) => void;
};

const PlaylistDetails: FC<PlaylistDetailsProps> = ({
  playlist,
  tracks,
  isSmart,
  loading,
  onGoBack,
  onPlayAll,
  onShuffleAll,
  onPlayTrack,
  onRemoveTrack,
}) => {
  const columns = [
    columnHelper.accessor("trackNumber", {
      header: "#",
      size: 20,
      cell: (info) => (
        <div style={{ position: "relative" }}>
          <div className="tracknumber">{info.row.index + 1}</div>
          <div
            className="floating-play"
            onClick={() => onPlayTrack(info.row.index)}
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
            width: "calc(100% - 20px)",
            maxWidth: "calc(100vw - 800px)",
            fontSize: 14,
            overflow: "hidden",
            textOverflow: "ellipsis",
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
    columnHelper.accessor("time", {
      header: "Time",
      size: 50,
      cell: (info) => info.getValue(),
    }),
    columnHelper.accessor("id", {
      header: "",
      size: 120,
      cell: (info) => (
        <div style={{ display: "flex", justifyContent: "flex-end", alignItems: "center", gap: 8 }}>
          {!isSmart && onRemoveTrack && (
            <button
              onClick={() => onRemoveTrack(info.row.original.id)}
              style={{
                background: "transparent",
                border: "none",
                cursor: "pointer",
                color: "#aaa",
                fontSize: 12,
                padding: "2px 6px",
              }}
              title="Remove from playlist"
            >
              Remove
            </button>
          )}
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
      <Sidebar active="playlists" />
      <MainView>
        <ControlBar />
        <div className="pl-4 pr-4 md:pl-[30px] md:pr-[30px] overflow-y-auto h-[var(--content-area-height)]">
          <button
            className="border-0 cursor-pointer flex items-center justify-center h-[30px] w-[30px] rounded-[15px] bg-[var(--theme-back-button)] mt-[26px] mb-[46px] absolute z-[1]"
            onClick={onGoBack}
          >
            <div style={{ marginTop: 2 }}>
              <ArrowBack color="var(--theme-icon)" />
            </div>
          </button>
          {loading && (
            <div style={{ marginTop: 60, marginBottom: 100 }}>
              <DetailHeaderSkeleton />
              <TrackListSkeleton />
            </div>
          )}
          {!loading && <div style={{ marginBottom: 100 }}>
            <div className="flex flex-col md:flex-row md:items-center mb-5 mt-[90px] gap-5">
              <div
                className="h-[160px] w-[160px] md:h-[240px] md:w-[240px] rounded-[6px] bg-[var(--theme-cover)] flex items-center justify-center flex-shrink-0 self-center md:self-auto"
                style={playlist?.image ? { backgroundImage: `url(${playlist.image})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}}
              >
                {!playlist?.image && <Music size={48} color="#bbb" />}
              </div>
              <div className="flex flex-col md:ml-[26px] md:h-[240px] justify-center text-center md:text-left">
                <div className="text-[24px] md:text-[32px] font-[RockfordSansBold]">{playlist?.name}</div>
                {playlist?.description && (
                  <div className="text-sm text-[#555] mt-2">
                    {playlist.description}
                  </div>
                )}
                <div className="mt-[25px] font-normal text-sm">{tracks.length} TRACKS</div>
                <div className="flex flex-row items-center mt-5 justify-center md:justify-start">
                  <Button onClick={onPlayAll} kind="primary">
                    <div className="flex flex-row items-center">
                      <Play small color="#fff" />
                      <div style={{ marginLeft: 7 }}>Play</div>
                    </div>
                  </Button>
                  <div className="w-5" />
                  <Button onClick={onShuffleAll} kind="secondary">
                    <div className="flex flex-row items-center">
                      <Shuffle color="#6F00FF" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </div>
                  </Button>
                </div>
              </div>
            </div>
            {tracks.length > 0 && (
              <Table columns={columns as any} tracks={tracks} />
            )}
          </div>}
        </div>
      </MainView>
    </div>
  );
};

export default PlaylistDetails;

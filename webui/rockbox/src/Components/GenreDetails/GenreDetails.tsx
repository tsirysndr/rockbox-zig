/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Link } from "react-router-dom";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar/Sidebar";
import ControlBar from "../ControlBar";
import { colorForSeed } from "../Genres/Genres";
import ArrowBack from "../Icons/ArrowBack";
import Shuffle from "../Icons/Shuffle";
import Play from "../Icons/Play";
import Button from "../Button";
import Album from "../Album";
import ArtistIcon from "../Icons/Artist";
import type {
  GenreAlbum,
  GenreArtist,
  GenreTrack,
} from "../../Hooks/useGenres";
import { createColumnHelper } from "@tanstack/react-table";
import Table from "../Table";
import ContextMenu from "../ContextMenu";

type Track = GenreTrack & { time: string };
const columnHelper = createColumnHelper<Track>();

export type GenreDetailsProps = {
  id: string;
  name: string;
  trackCount: number;
  tracks: Track[];
  albums: (GenreAlbum & { cover: string })[];
  artists: GenreArtist[];
  loading?: boolean;
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onPlayTrack: (idx: number) => void;
  onGoBack: () => void;
};

const GenreDetails: FC<GenreDetailsProps> = (props) => {
  const heroBg = colorForSeed(props.id || props.name);

  const columns = [
    columnHelper.accessor("title", {
      header: "Title",
      cell: (info) => (
        <div
          style={{
            minWidth: 150,
            maxWidth: 320,
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            color: "var(--theme-text)",
            cursor: "pointer",
          }}
          onClick={() => props.onPlayTrack(info.row.index)}
        >
          {info.getValue()}
        </div>
      ),
    }),
    columnHelper.accessor("artist", {
      header: "Artist",
      cell: (info) => (
        <div style={{ fontSize: 14, color: "var(--theme-text)" }}>
          {info.getValue()}
        </div>
      ),
    }),
    columnHelper.accessor("album", {
      header: "Album",
      cell: (info) => (
        <div style={{ fontSize: 14, color: "var(--theme-text)" }}>
          {info.getValue()}
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
      size: 60,
      cell: (info) => (
        <ContextMenu
          track={{
            id: info.row.original.id ?? "",
            title: info.row.original.title,
            artist: info.row.original.artist,
            time: info.row.original.time,
            cover: info.row.original.albumArt ?? undefined,
            path: info.row.original.path,
          }}
        />
      ),
    }),
  ];

  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="genres" />
      <div className="flex flex-1 flex-col w-full md:w-[calc(100%-240px)] bg-[var(--theme-background)]">
        <ControlBar />
        <div className="pl-4 pr-4 md:pl-[30px] md:pr-[30px] overflow-y-auto h-[var(--content-area-height)]">
          <button
            className="border-0 cursor-pointer flex items-center justify-center h-[30px] w-[30px] rounded-[15px] bg-[var(--theme-back-button)] mt-[26px] mb-[18px] absolute z-[1]"
            onClick={() => props.onGoBack()}
          >
            <ArrowBack color="var(--theme-icon)" />
          </button>

          <div
            style={{
              background: `linear-gradient(135deg, ${heroBg}, rgba(0,0,0,0.4))`,
            }}
            className="relative h-[220px] rounded-[12px] overflow-hidden mt-[30px] mb-6 flex items-end p-[22px_28px] text-white"
          >
            <div style={{ position: "relative", zIndex: 1 }}>
              <div className="text-xs font-semibold uppercase tracking-[2px] opacity-85">
                Genre
              </div>
              <div className="font-[RockfordSansMedium] text-[38px] font-bold mt-[6px]">
                {props.name}
              </div>
              <div className="text-xs mt-2 opacity-85">
                {props.trackCount} tracks · {props.albums.length} albums ·{" "}
                {props.artists.length} artists
              </div>
            </div>
            <div className="absolute right-[-10px] bottom-[-22px] font-[RockfordSansMedium] text-[110px] font-bold opacity-[0.18] rotate-[-12deg] pointer-events-none">
              {props.name}
            </div>
          </div>

          <div className="flex flex-row items-center gap-[14px]">
            <Button
              onClick={props.onPlayAll}
              kind="primary"
              disabled={props.tracks.length === 0}
            >
              <div className="flex flex-row items-center">
                <Play small color="#fff" />
                <div style={{ marginLeft: 7 }}>Play</div>
              </div>
            </Button>
            <Button
              onClick={props.onShuffleAll}
              kind="secondary"
              disabled={props.tracks.length === 0}
            >
              <div className="flex flex-row items-center">
                <Shuffle color="#6F00FF" />
                <div style={{ marginLeft: 7 }}>Shuffle</div>
              </div>
            </Button>
          </div>

          {props.tracks.length > 0 && (
            <>
              <div className="mt-[30px] mb-[14px] text-[20px] font-semibold">
                Popular tracks
              </div>
              <Table
                columns={columns as any}
                tracks={props.tracks.slice(0, 10) as any}
              />
            </>
          )}

          {props.albums.length > 0 && (
            <>
              <div className="mt-[30px] mb-[14px] text-[20px] font-semibold">
                Albums
              </div>
              <div style={{ marginBottom: 30 }}>
                <Grid
                  gridColumns={[2, 4, 5]}
                  gridMargins={[0, 0, 0]}
                  gridGutters={[20, 20, 20]}
                >
                  {props.albums.map((a) => (
                    <Cell key={a.id}>
                      <Album album={a} />
                    </Cell>
                  ))}
                </Grid>
              </div>
            </>
          )}

          {props.artists.length > 0 && (
            <>
              <div className="mt-[30px] mb-[14px] text-[20px] font-semibold">
                Artists
              </div>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(auto-fill, minmax(120px, 1fr))",
                  gap: 20,
                  marginBottom: 100,
                }}
              >
                {props.artists.map((artist) => (
                  <Link
                    key={artist.id}
                    to={`/artists/${artist.id}`}
                    className="flex flex-col items-center no-underline text-inherit cursor-pointer"
                  >
                    {artist.image ? (
                      <img
                        src={artist.image}
                        alt={artist.name}
                        className="w-[100px] h-[100px] rounded-full object-cover"
                      />
                    ) : (
                      <div className="w-[100px] h-[100px] rounded-full bg-[var(--theme-cover)] flex items-center justify-center">
                        <ArtistIcon width={42} height={42} color="#bbb" />
                      </div>
                    )}
                    <div className="mt-2 text-[13px] text-center max-w-[100px] overflow-hidden text-ellipsis whitespace-nowrap">
                      {artist.name}
                    </div>
                  </Link>
                ))}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default GenreDetails;

/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Link } from "react-router-dom";
import { LazyLoadImage } from "react-lazy-load-image-component";
import Sidebar from "../Sidebar/Sidebar";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Shuffle from "../Icons/Shuffle";
import Play from "../Icons/Play";
import Button from "../Button";
import { createColumnHelper } from "@tanstack/react-table";
import { Track } from "../../Types/track";
import Table from "../Table";
import { Cell, Grid } from "baseui/layout-grid";
import "./styles.css";
import ContextMenu from "../ContextMenu";
import Album from "../Album";
import TrackIcon from "../Icons/Track";
import ArtistIcon from "../Icons/Artist";
import ArtistHeaderSkeleton from "../Skeletons/ArtistHeaderSkeleton";
import TrackListSkeleton from "../Skeletons/TrackListSkeleton";
import AlbumCardSkeleton from "../Skeletons/AlbumCardSkeleton";

const columnHelper = createColumnHelper<Track>();

export type ArtistDetailsProps = {
  name: string;
  image?: string;
  tracks: Track[];
  albums: any[];
  loading?: boolean;
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onPlayAlbum: (album: any) => void;
  onLikeAlbum: (album: any) => void;
  onUnLikeAlbum: (album: any) => void;
  onLikeTrack: (track: any) => void;
  onUnlikeTrack: (track: any) => void;
  onGoBack: () => void;
  onPlayTrack: (position: number) => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = (props) => {
  const { image, loading } = props;
  const columns = [
    columnHelper.accessor("albumArt", {
      header: "Title",
      size: 48,
      cell: (info) => (
        <>
          {info.getValue() && (
            <div className="album-cover-container">
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
                <Play small color={info.getValue() ? "#fff" : "var(--theme-text)"} />
              </div>
            </div>
          )}
          {!info.getValue() && (
            <div className="album-cover-container">
              <div className="h-12 w-12 rounded-[4px] cursor-pointer bg-[var(--theme-cover)] flex justify-center items-center">
                <TrackIcon width={28} height={28} color="#a4a3a3" />
              </div>
              <div
                onClick={() => props.onPlayTrack(info.row.index)}
                className="floating-play"
              >
                <Play small color={info.getValue() ? "#fff" : "var(--theme-text)"} />
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
            width: "calc(100% - 20px)",
            maxWidth: "300px",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
          }}
          className="text-[var(--theme-text)]"
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
            maxWidth: "300px",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
            whiteSpace: "nowrap",
            cursor: "pointer",
          }}
          className="text-[var(--theme-text)]"
        >
          <Link
            className="text-[var(--theme-text)] no-underline font-[RockfordSansRegular] hover:underline"
            to={`/artists/${info.row.original.artistId}`}
          >
            {info.getValue()}
          </Link>
        </div>
      ),
    }),
    columnHelper.accessor("album", {
      header: "Album",
      cell: (info) => (
        <div
          style={{
            width: "calc(100% - 20px)",
            maxWidth: "250px",
            fontSize: 14,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            cursor: "pointer",
          }}
          className="text-[var(--theme-text)] hidden md:block"
        >
          <Link
            className="text-[var(--theme-text)] no-underline font-[RockfordSansRegular] hover:underline"
            to={`/albums/${info.row.original.albumId}`}
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
      <Sidebar active="artists" />
      <div className="flex flex-1 flex-col w-full md:w-[calc(100%-240px)] bg-[var(--theme-background)]">
        <ControlBar />
        <div className="pl-4 pr-4 md:pl-[30px] md:pr-[30px] overflow-y-auto h-[var(--content-area-height)]">
          <button
            className="border-0 cursor-pointer flex items-center justify-center h-[30px] w-[30px] rounded-[15px] bg-[var(--theme-back-button)] mt-[26px] mb-[46px] absolute z-[1]"
            onClick={() => props.onGoBack()}
          >
            <div style={{ marginTop: 2 }}>
              <ArrowBack color="var(--theme-icon)" />
            </div>
          </button>
          {loading && (
            <div style={{ marginTop: 20, marginBottom: 100 }}>
              <ArtistHeaderSkeleton />
              <div style={{ marginTop: 40 }}>
                <TrackListSkeleton />
              </div>
              <div style={{ marginTop: 40 }}>
                <Grid
                  gridColumns={[2, 4, 5]}
                  gridMargins={[0, 0, 0]}
                  gridGutters={[20, 20, 20]}
                >
                  {Array.from({ length: 5 }).map((_, i) => (
                    <Cell key={i}>
                      <AlbumCardSkeleton />
                    </Cell>
                  ))}
                </Grid>
              </div>
            </div>
          )}
          {!loading && <><div className="flex flex-col md:flex-row md:items-end gap-4 md:gap-6 mt-[30px] mb-8 items-center">
            {image ? (
              <img
                className="w-[120px] h-[120px] md:w-[160px] md:h-[160px] rounded-full object-cover flex-shrink-0"
                src={image}
                alt={props.name}
              />
            ) : (
              <div className="w-[120px] h-[120px] md:w-[160px] md:h-[160px] rounded-full bg-[var(--theme-cover)] flex items-center justify-center flex-shrink-0">
                <ArtistIcon width={48} height={48} color="#bbb" />
              </div>
            )}
            <div className="text-center md:text-left">
              <div className="font-[RockfordSansMedium] text-[24px] md:text-[30px] mb-1">
                {props.name}
              </div>
            </div>
          </div>
          <div className="flex flex-row items-center justify-center md:justify-start">
            <Button onClick={props.onPlayAll} kind="primary">
              <div className="flex flex-row items-center">
                <Play small color="#fff" />
                <div style={{ marginLeft: 7 }}>Play</div>
              </div>
            </Button>
            <div className="w-5" />
            <Button onClick={props.onShuffleAll} kind="secondary">
              <div className="flex flex-row items-center">
                <Shuffle color="#6F00FF" />
                <div style={{ marginLeft: 7 }}>Shuffle</div>
              </div>
            </Button>
          </div>
          <div className="mt-[30px] text-[20px] font-semibold">Tracks</div>
          <Table columns={columns as any} tracks={props.tracks} />
          <div className="mt-[30px] text-[20px] font-semibold" style={{ marginTop: 50 }}>Albums</div>
          <div style={{ marginBottom: 100 }}>
            <Grid
              gridColumns={[2, 4, 5]}
              gridMargins={[0, 0, 0]}
              gridGutters={[20, 20, 20]}
            >
              {props.albums.map((item) => (
                <Cell key={item.id}>
                  <Album album={item} />
                </Cell>
              ))}
            </Grid>
          </div>
          </>}
        </div>
      </div>
    </div>
  );
};

export default ArtistDetails;

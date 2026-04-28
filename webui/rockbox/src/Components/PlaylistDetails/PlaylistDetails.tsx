/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { useTheme } from "@emotion/react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import MainView from "../MainView/MainView";
import {
  Container,
  ContentWrapper,
  Header,
  CoverArt,
  PlaylistInfos,
  PlaylistTitle,
  PlaylistDescription,
  TrackCount,
  ButtonGroup,
  Separator,
  BackButton,
  Label,
  Link,
} from "./styles";
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
  const theme = useTheme();
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
            <Play color={theme.colors.icon} small />
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
            color: theme.colors.text,
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
            color: theme.colors.text,
          }}
        >
          <Link to={`/artists/${info.row.original.artistId}`}>
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
    <Container>
      <Sidebar active="playlists" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={onGoBack}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={theme.colors.icon} />
            </div>
          </BackButton>
          {loading && (
            <div style={{ marginTop: 60, marginBottom: 100 }}>
              <DetailHeaderSkeleton />
              <TrackListSkeleton />
            </div>
          )}
          {!loading && <div style={{ marginBottom: 100 }}>
            <Header>
              <CoverArt image={playlist?.image}>
                {!playlist?.image && <Music size={64} color="#bbb" />}
              </CoverArt>
              <PlaylistInfos>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "center",
                    height: "calc(240px - 12px)",
                  }}
                >
                  <PlaylistTitle>{playlist?.name}</PlaylistTitle>
                  {playlist?.description && (
                    <PlaylistDescription>
                      {playlist.description}
                    </PlaylistDescription>
                  )}
                  <TrackCount>{tracks.length} TRACKS</TrackCount>
                </div>
                <ButtonGroup>
                  <Button onClick={onPlayAll} kind="primary">
                    <Label>
                      <Play small color="#fff" />
                      <div style={{ marginLeft: 7 }}>Play</div>
                    </Label>
                  </Button>
                  <Separator />
                  <Button onClick={onShuffleAll} kind="secondary">
                    <Label>
                      <Shuffle color="#6F00FF" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </ButtonGroup>
              </PlaylistInfos>
            </Header>
            {tracks.length > 0 && (
              <Table columns={columns as any} tracks={tracks} />
            )}
          </div>}
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default PlaylistDetails;

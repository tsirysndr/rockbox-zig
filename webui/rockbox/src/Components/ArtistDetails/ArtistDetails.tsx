/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { useTheme } from "@emotion/react";
import Sidebar from "../Sidebar/Sidebar";
import ControlBar from "../ControlBar";
import {
  ArtistHeader,
  ArtistPicture,
  ArtistPicturePlaceholder,
  BackButton,
  ButtonGroup,
  Container,
  ContentWrapper,
  Label,
  MainView,
  Name,
  Separator,
  Title,
  Link,
  SmallAlbumCover,
  AlbumCoverAlt,
} from "./styles";
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
  const theme = useTheme();
  const columns = [
    columnHelper.accessor("albumArt", {
      header: "Title",
      size: 48,
      cell: (info) => (
        <>
          {info.getValue() && (
            <div className="album-cover-container">
              <SmallAlbumCover
                src={info.getValue()!}
                alt="album art"
                effect="blur"
              />
              <div
                onClick={() => props.onPlayTrack(info.row.index)}
                className="floating-play"
              >
                <Play small color={info.getValue() ? "#fff" : theme.colors.text} />
              </div>
            </div>
          )}
          {!info.getValue() && (
            <div className="album-cover-container">
              <AlbumCoverAlt>
                <TrackIcon width={28} height={28} color="#a4a3a3" />
              </AlbumCoverAlt>
              <div
                onClick={() => props.onPlayTrack(info.row.index)}
                className="floating-play"
              >
                <Play small color={info.getValue() ? "#fff" : theme.colors.text} />
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
            maxWidth: "300px",
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
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
    columnHelper.accessor("album", {
      header: "Album",
      cell: (info) => (
        <div
          style={{
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
          <Link to={`/albums/${info.row.original.albumId}`}>
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
        <ButtonGroup
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
        </ButtonGroup>
      ),
    }),
  ];

  return (
    <Container>
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => props.onGoBack()}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={theme.colors.icon} />
            </div>
          </BackButton>
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
          {!loading && <><ArtistHeader>
            {image ? (
              <ArtistPicture src={image} alt={props.name} />
            ) : (
              <ArtistPicturePlaceholder>
                <ArtistIcon width={64} height={64} color="#bbb" />
              </ArtistPicturePlaceholder>
            )}
            <div>
              <Name>{props.name}</Name>
            </div>
          </ArtistHeader>
          <ButtonGroup>
            <Button onClick={props.onPlayAll} kind="primary">
              <Label>
                <Play small color="#fff" />
                <div style={{ marginLeft: 7 }}>Play</div>
              </Label>
            </Button>
            <Separator />
            <Button onClick={props.onShuffleAll} kind="secondary">
              <Label>
                <Shuffle color="#6F00FF" />
                <div style={{ marginLeft: 7 }}>Shuffle</div>
              </Label>
            </Button>
          </ButtonGroup>
          <Title>Tracks</Title>
          <Table columns={columns as any} tracks={props.tracks} />
          <Title style={{ marginTop: 50 }}>Albums</Title>
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
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default ArtistDetails;

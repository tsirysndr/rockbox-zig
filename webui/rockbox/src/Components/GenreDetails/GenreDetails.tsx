/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { useTheme } from "@emotion/react";
import { Cell, Grid } from "baseui/layout-grid";
import Sidebar from "../Sidebar/Sidebar";
import ControlBar from "../ControlBar";
import {
  ArtistImage,
  ArtistImagePlaceholder,
  ArtistName,
  ArtistThumb,
  BackButton,
  ButtonGroup,
  Container,
  ContentWrapper,
  GenreDecoration,
  GenreHero,
  GenreLabel,
  GenreName,
  GenreStats,
  Label,
  MainView,
  SectionTitle,
} from "./styles";
import { colorForSeed } from "../Genres/styles";
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
  const theme = useTheme();
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
            color: theme.colors.text,
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
        <div style={{ fontSize: 14, color: theme.colors.text }}>
          {info.getValue()}
        </div>
      ),
    }),
    columnHelper.accessor("album", {
      header: "Album",
      cell: (info) => (
        <div style={{ fontSize: 14, color: theme.colors.text }}>
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
    <Container>
      <Sidebar active="genres" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => props.onGoBack()}>
            <ArrowBack color={theme.colors.icon} />
          </BackButton>

          <GenreHero bg={heroBg}>
            <div style={{ position: "relative", zIndex: 1 }}>
              <GenreLabel>Genre</GenreLabel>
              <GenreName>{props.name}</GenreName>
              <GenreStats>
                {props.trackCount} tracks · {props.albums.length} albums ·{" "}
                {props.artists.length} artists
              </GenreStats>
            </div>
            <GenreDecoration>{props.name}</GenreDecoration>
          </GenreHero>

          <ButtonGroup>
            <Button
              onClick={props.onPlayAll}
              kind="primary"
              disabled={props.tracks.length === 0}
            >
              <Label>
                <Play small color="#fff" />
                <div style={{ marginLeft: 7 }}>Play</div>
              </Label>
            </Button>
            <Button
              onClick={props.onShuffleAll}
              kind="secondary"
              disabled={props.tracks.length === 0}
            >
              <Label>
                <Shuffle color="#6F00FF" />
                <div style={{ marginLeft: 7 }}>Shuffle</div>
              </Label>
            </Button>
          </ButtonGroup>

          {props.tracks.length > 0 && (
            <>
              <SectionTitle>Popular tracks</SectionTitle>
              <Table
                columns={columns as any}
                tracks={props.tracks.slice(0, 10) as any}
              />
            </>
          )}

          {props.albums.length > 0 && (
            <>
              <SectionTitle>Albums</SectionTitle>
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
              <SectionTitle>Artists</SectionTitle>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(auto-fill, minmax(120px, 1fr))",
                  gap: 20,
                  marginBottom: 100,
                }}
              >
                {props.artists.map((artist) => (
                  <ArtistThumb key={artist.id} to={`/artists/${artist.id}`}>
                    {artist.image ? (
                      <ArtistImage src={artist.image} alt={artist.name} />
                    ) : (
                      <ArtistImagePlaceholder>
                        <ArtistIcon width={42} height={42} color="#bbb" />
                      </ArtistImagePlaceholder>
                    )}
                    <ArtistName>{artist.name}</ArtistName>
                  </ArtistThumb>
                ))}
              </div>
            </>
          )}
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default GenreDetails;

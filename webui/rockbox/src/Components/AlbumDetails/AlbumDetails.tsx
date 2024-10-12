/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  Container,
  MainView,
  AlbumCover,
  ContentWrapper,
  AlbumTitle,
  Header,
  AlbumInfos,
  Artist,
  Tracks,
  Year,
  ButtonGroup,
  Separator,
  BackButton,
  IconButton,
  Hover,
  Label,
  Link,
} from "./styles";
import Button from "../Button";
import ArrowBack from "../Icons/ArrowBack";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import Add from "../Icons/Add";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import Table from "../Table";
import { Track } from "../../Types/track";
import { Album } from "../../Hooks/GraphQL";
import AlbumArt from "../../Assets/albumart.svg";

const columnHelper = createColumnHelper<Track>();

export type AlbumDetailsProps = {
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onGoBack: () => void;
  onLike: (track: string) => void;
  onUnlike: (track: string) => void;
  tracks: Track[];
  album?: Album | null;
  volumes: Track[][];
};

const AlbumDetails: FC<AlbumDetailsProps> = (props) => {
  const columns = [
    columnHelper.accessor("trackNumber", {
      header: "#",
      size: 20,
      cell: (info) => info.getValue(),
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
            color: "#000",
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
            color: "#000",
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
      size: 100,
      cell: (info) => (
        <ButtonGroup
          style={{ justifyContent: "flex-end", alignItems: "center" }}
        >
          <IconButton>
            <Hover>
              <EllipsisHorizontal size={24} />
            </Hover>
          </IconButton>
          <IconButton>
            <Add color="#000" size={24} />
          </IconButton>
          <IconButton onClick={() => props.onLike(info.row.original.id)}>
            <HeartOutline color="#000" />
          </IconButton>
        </ButtonGroup>
      ),
    }),
  ];

  return (
    <Container>
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => props.onGoBack()}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={"#000"} />
            </div>
          </BackButton>
          <div style={{ marginBottom: 100 }}>
            <Header>
              <AlbumCover src={props.album?.albumArt || AlbumArt} />
              <AlbumInfos>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "center",
                    height: "calc(240px - 12px)",
                  }}
                >
                  <AlbumTitle>{props.album?.title}</AlbumTitle>
                  <Artist to={`/artists/${props.album?.artistId}`}>
                    {props.album?.artist}
                  </Artist>
                  <Tracks>
                    {props.tracks.length || props.volumes.flat().length} TRACKS
                  </Tracks>
                  <Year>{props.album?.year}</Year>
                </div>
                <ButtonGroup>
                  <Button onClick={() => props.onPlayAll()} kind="primary">
                    <Label>
                      <Play small color="#fff" />
                      <div style={{ marginLeft: 7 }}>Play</div>
                    </Label>
                  </Button>
                  <Separator />
                  <Button onClick={() => props.onShuffleAll()} kind="secondary">
                    <Label>
                      <Shuffle color="#fe099c" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </ButtonGroup>
              </AlbumInfos>
            </Header>
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
          </div>
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default AlbumDetails;

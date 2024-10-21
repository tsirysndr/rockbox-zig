/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC, useRef } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import MainView from "../MainView";
import {
  AlbumCover,
  AlbumCoverAlt,
  ButtonGroup,
  Container,
  ContentWrapper,
  FilterContainer,
  Link,
  Title,
  Separator,
  Label,
  HeaderWrapper,
} from "./styles";
import { Track } from "../../Types/track";
import Table from "../VirtualizedTable";
import Filter from "../Filter";
import TrackIcon from "../Icons/Track";
import "./styles.css";
import ContextMenu from "../ContextMenu";
import Button from "../Button";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";

const columnHelper = createColumnHelper<Track>();

export type TracksProps = {
  tracks: Track[];
  onPlayTrack: (id: string) => void;
  onPlayAll: () => void;
  onShuffleAll: () => void;
};

const Likes: FC<TracksProps> = (props) => {
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
            height: 48,
            marginTop: -10,
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
            <div className="album-cover-container">
              <AlbumCover
                src={info.getValue()!}
                alt="album art"
                effect="blur"
              />
              <div
                onClick={() => props.onPlayTrack(info.row.original.id)}
                className="floating-play"
              >
                <Play small color={info.getValue() ? "#fff" : "#000"} />
              </div>
            </div>
          )}
          {!info.getValue() && (
            <div className="album-cover-container">
              <AlbumCoverAlt>
                <TrackIcon width={28} height={28} color="#a4a3a3" />
              </AlbumCoverAlt>
              <div
                onClick={() => props.onPlayTrack(info.row.original.id)}
                className="floating-play"
              >
                <Play small color={info.getValue() ? "#fff" : "#000"} />
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
            maxWidth: 170,
            fontSize: 14,
            textOverflow: "ellipsis",
            overflow: "hidden",
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
            color: "#000",
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
      <Sidebar active="likes" />
      <MainView>
        <ControlBar />
        <ContentWrapper ref={containerRef}>
          <Title>Likes</Title>
          {props.tracks.length > 0 && (
            <>
              <HeaderWrapper>
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
                      <Shuffle color="#fe099c" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </ButtonGroup>
                <FilterContainer>
                  <Filter placeholder="Search song" onChange={() => {}} />
                </FilterContainer>
              </HeaderWrapper>
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
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Likes;

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
  Hover,
  IconButton,
  Link,
  Title,
} from "./styles";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import Add from "../Icons/Add";
import HeartOutline from "../Icons/HeartOutline";
import { Track } from "../../Types/track";
import Table from "../VirtualizedTable";
import Filter from "../Filter";
import TrackIcon from "../Icons/Track";
import { Play } from "@styled-icons/ionicons-sharp";
import "./styles.css";

const columnHelper = createColumnHelper<Track>();

export type TracksProps = {
  tracks: Track[];
  onPlayTrack: (id: string) => void;
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
                <Play size={16} color={info.getValue() ? "#fff" : "#000"} />
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
                <Play size={16} color={info.getValue() ? "#fff" : "#000"} />
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
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      cell: (_info) => (
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
          <IconButton>
            <HeartOutline color="#000" />
          </IconButton>
        </ButtonGroup>
      ),
    }),
  ];
  return (
    <Container>
      <Sidebar active="songs" />
      <MainView>
        <ControlBar />
        <ContentWrapper ref={containerRef}>
          <Title>Songs</Title>
          <FilterContainer>
            <Filter placeholder="Search song" onChange={() => {}} />
          </FilterContainer>
          <div style={{ marginBottom: 60 }}>
            <Table
              columns={columns as any}
              tracks={props.tracks}
              containerRef={containerRef}
            />
          </div>
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Tracks;

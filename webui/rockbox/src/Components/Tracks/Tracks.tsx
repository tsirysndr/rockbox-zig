/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  AlbumCover,
  ButtonGroup,
  Container,
  ContentWrapper,
  FilterContainer,
  Hover,
  IconButton,
  Link,
  MainView,
  Title,
} from "./styles";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import Add from "../Icons/Add";
import HeartOutline from "../Icons/HeartOutline";
import { Track } from "../../Types/track";
import Table from "../VirtualizedTable";
import { tracks } from "./mocks";
import Filter from "../Filter";
import "./styles.css";

const columnHelper = createColumnHelper<Track>();
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
        }}
      >
        {info.getValue()}
      </div>
    ),
  }),
  columnHelper.accessor("albumArt", {
    header: "Title",
    size: 54,
    cell: (info) => <AlbumCover src={info.getValue()} alt="album art" />,
  }),
  columnHelper.accessor("title", {
    header: "",
    cell: (info) => (
      <div
        style={{
          minWidth: 150,
          width: "calc(100% - 20px)",
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
          fontSize: 14,
          textOverflow: "ellipsis",
          overflow: "hidden",
          whiteSpace: "nowrap",
          cursor: "pointer",
          color: "#000",
        }}
      >
        <Link href="#">{info.getValue()}</Link>
      </div>
    ),
  }),
  columnHelper.accessor("album", {
    header: "Album",
    cell: (info) => (
      <div
        style={{
          minWidth: 150,
          width: "calc(100% - 20px)",
          fontSize: 14,
          textOverflow: "ellipsis",
          overflow: "hidden",
          whiteSpace: "nowrap",
          cursor: "pointer",
          color: "#000",
        }}
      >
        <Link href="#">{info.getValue()}</Link>
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
      <ButtonGroup style={{ justifyContent: "flex-end", alignItems: "center" }}>
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

const Tracks: FC = () => {
  return (
    <Container>
      <Sidebar active="songs" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <Title>Songs</Title>
          <FilterContainer>
            <Filter placeholder="Search song" onChange={() => {}} />
          </FilterContainer>
          <Table columns={columns as any} tracks={tracks} />
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Tracks;

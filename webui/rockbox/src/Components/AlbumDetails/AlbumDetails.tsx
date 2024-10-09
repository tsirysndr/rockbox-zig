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
} from "./styles";
import Button from "../Button";
import ArrowBack from "../Icons/ArrowBack";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import Add from "../Icons/Add";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import Table from "../Table";
import { tracks } from "./mocks";
import { Track } from "../../Types/track";

const columnHelper = createColumnHelper<Track>();
const columns = [
  columnHelper.accessor("trackNumber", {
    header: "#",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("title", {
    header: "Title",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("artist", {
    header: "Artist",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("time", {
    header: "Time",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("id", {
    header: "",
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

const AlbumDetails: FC = () => {
  return (
    <Container>
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => {}}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={"#000"} />
            </div>
          </BackButton>
          <Header>
            <AlbumCover src="https://resources.tidal.com/images/f6f5f0a6/dc95/4561/9ca6/6ba1e0f6a062/320x320.jpg" />
            <AlbumInfos>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "center",
                  height: "calc(240px - 12px)",
                }}
              >
                <AlbumTitle>One Cold Night (Live)</AlbumTitle>
                <Artist>Seether</Artist>
                <Tracks>13 TRACKS</Tracks>
                <Year>2006</Year>
              </div>
              <ButtonGroup>
                <Button onClick={() => {}} kind="primary">
                  <Label>
                    <Play small color="#fff" />
                    <div style={{ marginLeft: 7 }}>Play</div>
                  </Label>
                </Button>
                <Separator />
                <Button onClick={() => {}} kind="secondary">
                  <Label>
                    <Shuffle color="#fe099c" />
                    <div style={{ marginLeft: 7 }}>Shuffle</div>
                  </Label>
                </Button>
              </ButtonGroup>
            </AlbumInfos>
          </Header>
          <Table columns={columns as any} tracks={tracks} />
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default AlbumDetails;

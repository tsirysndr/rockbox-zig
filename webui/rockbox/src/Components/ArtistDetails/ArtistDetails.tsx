/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  SmallAlbumCover,
  AlbumTitle,
  Artist,
  BackButton,
  ButtonGroup,
  Container,
  ContentWrapper,
  Hover,
  IconButton,
  Label,
  MainView,
  Name,
  Separator,
  Title,
  Year,
  AlbumCover,
} from "./styles";
import ArrowBack from "../Icons/ArrowBack";
import Shuffle from "../Icons/Shuffle";
import Play from "../Icons/Play";
import Button from "../Button";
import { createColumnHelper } from "@tanstack/react-table";
import { Track } from "../../Types/track";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import HeartOutline from "../Icons/HeartOutline";
import Add from "../Icons/Add";
import Table from "../Table";
import AlbumArt from "../../Assets/albumart.png";
import { Cell, Grid } from "baseui/layout-grid";

const columnHelper = createColumnHelper<Track>();
const columns = [
  columnHelper.accessor("cover", {
    header: "Title",
    size: 48,
    cell: (info) => (
      <SmallAlbumCover src={info.getValue() || AlbumArt} alt="album art" />
    ),
  }),
  columnHelper.accessor("title", {
    header: "",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("artist", {
    header: "Artist",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("album", {
    header: "Album",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("time", {
    header: "Time",
    size: 50,
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

export type ArtistDetailsProps = {
  name: string;
  tracks: Track[];
  albums: any[];
  onClickAlbum: (album: any) => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = (props) => {
  return (
    <Container>
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => {}}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={"#000"} />
            </div>
          </BackButton>
          <Name>{props.name}</Name>
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
          <Title>Tracks</Title>
          <Table columns={columns as any} tracks={props.tracks} />
          <Title style={{ marginBottom: 20, marginTop: 50 }}>Albums</Title>
          <div style={{ marginBottom: 100 }}>
            <Grid gridColumns={[2, 4, 5]} gridMargins={[8, 10, 12]}>
              {props.albums.map((item) => (
                <Cell key={item.id}>
                  <AlbumCover
                    src={item.cover ? item.cover : AlbumArt}
                    onClick={() => props.onClickAlbum(item)}
                    effect="opacity"
                  />
                  <AlbumTitle onClick={() => props.onClickAlbum(item)}>
                    {item.title}
                  </AlbumTitle>
                  <Artist>{item.artist}</Artist>
                  <Year>{item.year}</Year>
                </Cell>
              ))}
            </Grid>
          </div>
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default ArtistDetails;

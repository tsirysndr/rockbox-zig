/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import Sidebar from "../Sidebar/Sidebar";
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
  Label,
  MainView,
  Name,
  Separator,
  Title,
  Year,
  AlbumCover,
  Link,
  AlbumFooterMenu,
  FloatingButton,
  NoAlbumCover,
} from "./styles";
import ArrowBack from "../Icons/ArrowBack";
import Shuffle from "../Icons/Shuffle";
import Play from "../Icons/Play";
import Button from "../Button";
import { createColumnHelper } from "@tanstack/react-table";
import { Track } from "../../Types/track";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import HeartOutline from "../Icons/HeartOutline";
import Table from "../Table";
import AlbumArt from "../../Assets/albumart.svg";
import { Cell, Grid } from "baseui/layout-grid";
import { Link as RouterLink } from "react-router-dom";
import "./styles.css";
import ContextMenu from "../ContextMenu";

const columnHelper = createColumnHelper<Track>();
const columns = [
  columnHelper.accessor("albumArt", {
    header: "Title",
    size: 48,
    cell: (info) => (
      <SmallAlbumCover src={info.getValue() || AlbumArt} alt="album art" />
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
          maxWidth: "300px",
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
      <ButtonGroup style={{ justifyContent: "flex-end", alignItems: "center" }}>
        <ContextMenu
          track={{
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

export type ArtistDetailsProps = {
  name: string;
  tracks: Track[];
  albums: any[];
  onPlayAll: () => void;
  onShuffleAll: () => void;
  onPlayAlbum: (album: any) => void;
  onLikeAlbum: (album: any) => void;
  onUnLikeAlbum: (album: any) => void;
  onLikeTrack: (track: any) => void;
  onUnlikeTrack: (track: any) => void;
  onGoBack: () => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = (props) => {
  return (
    <Container>
      <Sidebar active="artists" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => props.onGoBack()}>
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
            <Grid
              gridColumns={[2, 4, 5]}
              gridMargins={[0, 0, 0]}
              gridGutters={[20, 20, 20]}
            >
              {props.albums.map((item) => (
                <Cell key={item.id}>
                  <div style={{ position: "relative", width: "100%" }}>
                    <Hover>
                      <AlbumFooterMenu>
                        <div
                          style={{
                            backgroundColor: "#ffffffbc",
                            height: 40,
                            width: 40,
                            borderRadius: 20,
                            display: "flex",
                            justifyContent: "center",
                            alignItems: "center",
                          }}
                        >
                          <Play small color="#000" />
                        </div>
                        <FloatingButton>
                          <EllipsisHorizontal size={24} color="#fff" />
                        </FloatingButton>
                        <FloatingButton>
                          <HeartOutline color="#fff" />
                        </FloatingButton>
                      </AlbumFooterMenu>
                    </Hover>
                    <RouterLink to={`/albums/${item.id}`}>
                      {item.albumArt && (
                        <AlbumCover src={item.albumArt} effect="opacity" />
                      )}
                      {!item.albumArt && <NoAlbumCover src={AlbumArt} />}
                    </RouterLink>
                  </div>
                  <AlbumTitle to={`/albums/${item.id}`}>
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

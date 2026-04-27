/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { useTheme } from "@emotion/react";
import {
  AlbumCover,
  AlbumFooterMenu,
  AlbumTitle,
  Artist,
  CoverWrapper,
  FloatingButton,
  Link,
  NoAlbumCover,
  Year,
} from "./styles";
import Play from "../Icons/Play";
import ContextMenu from "./ContextMenu";
import HeartOutline from "../Icons/HeartOutline";
import AlbumArt from "../../Assets/albumart.svg";
import Heart from "../Icons/Heart";

export type AlbumProps = {
  album: any;
  onPlay: (album: any) => void;
  onLike: (album: any) => void;
  onUnLike: (album: any) => void;
  liked?: boolean;
};

const Album: FC<AlbumProps> = (props) => {
  const theme = useTheme();
  return (
    <div style={{ width: "100%" }}>
      <CoverWrapper>
        <AlbumFooterMenu className="album-footer-menu">
          <div
            style={{
              backgroundColor: theme.colors.surface,
              height: 40,
              width: 40,
              borderRadius: 20,
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
            }}
            onClick={() => props.onPlay(props.album)}
          >
            <Play small color={theme.colors.icon} />
          </div>
          <ContextMenu item={props.album} />
          {!props.liked && (
            <FloatingButton onClick={() => props.onLike(props.album)}>
              <HeartOutline color="#fff" size={20} />
            </FloatingButton>
          )}
          {props.liked && (
            <FloatingButton onClick={() => props.onUnLike(props.album)}>
              <Heart color="#6F00FF" size={20} />
            </FloatingButton>
          )}
        </AlbumFooterMenu>
        <Link to={`/albums/${props.album.id}`}>
          {props.album.cover && (
            <AlbumCover src={props.album.cover} effect="opacity" />
          )}
          {!props.album.cover && <NoAlbumCover src={AlbumArt} />}
        </Link>
      </CoverWrapper>
      <Link to={`/albums/${props.album.id}`}>
        <AlbumTitle>{props.album.title}</AlbumTitle>
      </Link>
      <Artist to={`/artists/${props.album.artistId}`}>
        {props.album.artist}
      </Artist>
      <Year>{props.album.year}</Year>
    </div>
  );
};

export default Album;

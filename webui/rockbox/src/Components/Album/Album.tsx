/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import {
  AlbumCover,
  AlbumFooterMenu,
  AlbumTitle,
  Artist,
  FloatingButton,
  Hover,
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
  return (
    <div style={{ position: "relative", width: "100%" }}>
      <Hover>
        <AlbumFooterMenu>
          <div
            style={{
              backgroundColor: "#ffffffda",
              height: 40,
              width: 40,
              borderRadius: 20,
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
            }}
            onClick={() => props.onPlay(props.album)}
          >
            <Play small color="#000" />
          </div>
          <ContextMenu item={props.album} />
          {!props.liked && (
            <FloatingButton onClick={() => props.onLike(props.album)}>
              <HeartOutline color="#fff" size={20} />
            </FloatingButton>
          )}
          {props.liked && (
            <FloatingButton onClick={() => props.onUnLike(props.album)}>
              <Heart color="#fe09a3" size={20} />
            </FloatingButton>
          )}
        </AlbumFooterMenu>
      </Hover>
      <Link to={`/albums/${props.album.id}`}>
        {props.album.cover && (
          <AlbumCover src={props.album.cover} effect="opacity" />
        )}
        {!props.album.cover && <NoAlbumCover src={AlbumArt} />}
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

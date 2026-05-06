/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { Link } from "react-router-dom";
import { LazyLoadImage } from "react-lazy-load-image-component";
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
    <div style={{ width: "100%" }}>
      <div className="relative [&:hover_.album-footer-menu]:opacity-100 [&:hover_.album-footer-menu]:pointer-events-auto">
        <div className="album-footer-menu absolute bottom-0 left-[10px] h-[60px] flex flex-row items-center justify-between w-[calc(100%-20px)] opacity-0 pointer-events-none transition-opacity duration-150 z-[1]">
          <div
            style={{
              backgroundColor: "var(--theme-surface)",
              height: 40,
              width: 40,
              borderRadius: 20,
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
            }}
            onClick={() => props.onPlay(props.album)}
          >
            <Play small color="var(--theme-icon)" />
          </div>
          <ContextMenu item={props.album} />
          {!props.liked && (
            <button
              className="h-10 w-10 rounded-full flex justify-center items-center border-0 cursor-pointer bg-transparent hover:bg-[#434242b5]"
              onClick={() => props.onLike(props.album)}
            >
              <HeartOutline color="#fff" size={20} />
            </button>
          )}
          {props.liked && (
            <button
              className="h-10 w-10 rounded-full flex justify-center items-center border-0 cursor-pointer bg-transparent hover:bg-[#434242b5]"
              onClick={() => props.onUnLike(props.album)}
            >
              <Heart color="#6F00FF" size={20} />
            </button>
          )}
        </div>
        <Link to={`/albums/${props.album.id}`} className="no-underline">
          {props.album.cover && (
            <LazyLoadImage
              src={props.album.cover}
              effect="opacity"
              className="w-full rounded-[3px] cursor-pointer"
            />
          )}
          {!props.album.cover && (
            <img src={AlbumArt} className="w-full rounded-[3px] cursor-pointer" />
          )}
        </Link>
      </div>
      <Link to={`/albums/${props.album.id}`} className="no-underline">
        <div className="text-sm text-ellipsis overflow-hidden whitespace-nowrap cursor-pointer text-[var(--theme-text)]">
          {props.album.title}
        </div>
      </Link>
      <Link
        to={`/artists/${props.album.artistId}`}
        className="text-[var(--theme-secondary-text)] text-sm text-ellipsis overflow-hidden whitespace-nowrap cursor-pointer no-underline"
      >
        {props.album.artist}
      </Link>
      <div className="text-[var(--theme-secondary-text)] text-xs font-normal mb-14">
        {props.album.year}
      </div>
    </div>
  );
};

export default Album;

import { FC, useRef } from "react";
import { ProgressBar } from "baseui/progress-bar";
import { Link } from "react-router-dom";
import Track from "../../Icons/Track";
import { useTimeFormat } from "../../../Hooks/useFormat";
import { CurrentTrack as NowPlaying } from "../../../Types/track";
import _ from "lodash";
import HeartOutline from "../../Icons/HeartOutline";
import Heart from "../../Icons/Heart";

const progressbarStyles = {
  Progressbar: {
    BarContainer: {
      style: {
        marginLeft: 0,
        marginRight: 0,
      },
    },
    BarProgress: {
      style: () => ({
        backgroundColor: "#6F00FF",
      }),
    },
    Bar: {
      style: () => ({
        backgroundColor: "rgba(177, 178, 181, 0.218)",
      }),
    },
  },
};

export type CurrentTrackProps = {
  nowPlaying?: NowPlaying;
  liked?: boolean;
  onLike: (trackId: string) => void;
  onUnlike: (trackId: string) => void;
  onSeek: (time: number) => void;
};

const CurrentTrack: FC<CurrentTrackProps> = ({
  nowPlaying,
  liked,
  onLike,
  onUnlike,
  onSeek,
}) => {
  const progressbarRef = useRef<HTMLDivElement>(null);
  const { formatTime } = useTimeFormat();
  const album = `${nowPlaying?.artist} - ${nowPlaying?.album}`;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleClick = (e: any) => {
    if (progressbarRef.current) {
      const rect = progressbarRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left < 0 ? 0 : e.clientX - rect.left;
      const width = rect.width;
      const percentage = (x / width) * 100;
      const time = (percentage / 100) * nowPlaying!.duration;
      onSeek(Math.floor(time));
    }
  };

  return (
    <div className="flex flex-row min-w-[400px] flex-1 h-[60px] rounded-[4px] border border-track-border ml-5">
      {!nowPlaying?.cover && (
        <div className="w-[60px] h-[60px] bg-cover flex justify-center items-center rounded-tl-[4px] rounded-bl-[4px]">
          <Track color="#b1b2b5" height={28} width={28} />
        </div>
      )}
      {nowPlaying?.cover && (
        <img
          src={nowPlaying.cover}
          alt="Album cover"
          className="h-[60px] w-[60px] rounded-tl-[4px] rounded-bl-[4px]"
        />
      )}
      <div className="flex flex-col justify-center w-full relative text-sm">
        {(!nowPlaying || nowPlaying?.duration === 0) && (
          <div style={{ color: "#b1b2b5", textAlign: "center" }}>
            No track playing
          </div>
        )}
        {nowPlaying && nowPlaying?.duration > 0 && (
          <>
            <div style={{ display: "flex", flexDirection: "row" }}>
              <div className="w-[60px] flex items-center justify-center mr-[5px] opacity-0 hover:opacity-100" />
              <div className="text-center text-ellipsis overflow-hidden whitespace-nowrap w-[calc(100%-20px)] mx-[10px] text-text">
                {_.get(nowPlaying, "title.length", 0) > 75
                  ? `${nowPlaying.title?.substring(0, 75)}...`
                  : nowPlaying.title}
              </div>
              <div className="w-[60px] flex items-center justify-center mr-[5px] opacity-0 hover:opacity-100">
                {!liked && (
                  <div className="cursor-pointer" onClick={() => onLike(nowPlaying!.id!)}>
                    <HeartOutline color="var(--theme-icon)" />
                  </div>
                )}
                {liked && (
                  <div className="cursor-pointer" onClick={() => onUnlike(nowPlaying!.id!)}>
                    <Heart color="var(--theme-primary)" />
                  </div>
                )}
              </div>
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <div className="text-[10px] text-secondary-text font-[RockfordSansRegular] text-center w-[60px]">
                {formatTime(nowPlaying.progress)}
              </div>
              <div className="text-center text-secondary-text font-[RockfordSansLight] text-ellipsis overflow-hidden whitespace-nowrap w-[calc(100%-125px)]">
                {_.get(nowPlaying, "artist.length", 0) > 40
                  ? `${nowPlaying.artist?.substring(0, 40)}...`
                  : nowPlaying.artist}
                <span className="mx-2">-</span>
                <Link
                  to={`/albums/${nowPlaying.albumId}`}
                  className="no-underline text-inherit hover:underline"
                >
                  {album.length > 75
                    ? `${nowPlaying.album?.substring(0, 30)}...`
                    : nowPlaying.album}
                </Link>
              </div>
              <div className="text-[10px] text-secondary-text font-[RockfordSansRegular] text-center w-[60px]">
                {formatTime(nowPlaying.duration)}
              </div>
            </div>
            <div
              ref={progressbarRef}
              onClick={handleClick}
              className={`w-full absolute bottom-[-12px]${nowPlaying!.duration > 0 ? " cursor-pointer" : ""}`}
            >
              <ProgressBar
                value={
                  nowPlaying!.duration > 0
                    ? (nowPlaying!.progress / nowPlaying!.duration) * 100
                    : 0
                }
                overrides={progressbarStyles.Progressbar}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default CurrentTrack;

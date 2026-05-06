import { FC, useEffect, useState } from "react";
import Play from "../Icons/Play";
import Previous from "../Icons/Previous";
import Next from "../Icons/Next";
import Shuffle from "../Icons/Shuffle";
import Repeat from "../Icons/Repeat";
import CurrentTrack from "./CurrentTrack";
import RightMenu from "./RightMenu";
import Pause from "../Icons/Pause";
import { CurrentTrack as NowPlaying } from "../../Types/track";

export type ControlBarProps = {
  nowPlaying?: NowPlaying;
  onPlay: () => Promise<void>;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  liked?: boolean;
  onLike: (trackId: string) => void;
  onUnlike: (trackId: string) => void;
  onSeek: (time: number) => void;
  shuffle?: boolean;
  repeat?: boolean;
};

const ControlBar: FC<ControlBarProps> = (props) => {
  const [shuffle, setShuffle] = useState(props.shuffle);
  const [repeat, setRepeat] = useState(props.repeat);

  useEffect(() => {
    setShuffle(props.shuffle);
    setRepeat(props.repeat);
  }, [props.shuffle, props.repeat]);

  const onShuffle = () => {
    setShuffle(!shuffle);
    props.onShuffle();
  };

  const onRepeat = () => {
    setRepeat(!repeat);
    props.onRepeat();
  };

  return (
    <div className="flex flex-row h-[60px] mt-[5px] mb-5 pr-5">
      <div className="flex items-center justify-center flex-[0.3]">
        <div className="flex items-center justify-between flex-row w-[160px]">
          <button
            onClick={onShuffle}
            className={`bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60${shuffle ? " bg-[var(--theme-hover)]" : ""}`}
          >
            <Shuffle color="var(--theme-icon)" />
          </button>
          <button
            onClick={props.onPrevious}
            className="bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60"
          >
            <Previous color="var(--theme-icon)" />
          </button>
          {!props.nowPlaying?.isPlaying && (
            <button
              onClick={props.onPlay}
              className="bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60"
            >
              <Play color="var(--theme-icon)" />
            </button>
          )}
          {props.nowPlaying?.isPlaying && (
            <button
              onClick={props.onPause}
              className="bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60"
            >
              <Pause color="var(--theme-icon)" />
            </button>
          )}
          <button
            onClick={props.onNext}
            className="bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60"
          >
            <Next color="var(--theme-icon)" />
          </button>
          <button
            onClick={onRepeat}
            className={`bg-transparent cursor-pointer border-0 flex items-center justify-center p-2 rounded-[6px] hover:opacity-60${repeat ? " bg-[var(--theme-hover)]" : ""}`}
          >
            <Repeat color="var(--theme-icon)" />
          </button>
        </div>
      </div>
      <CurrentTrack
        nowPlaying={props.nowPlaying}
        liked={props.liked}
        onLike={props.onLike}
        onUnlike={props.onUnlike}
        onSeek={props.onSeek}
      />
      <RightMenu />
    </div>
  );
};

export default ControlBar;

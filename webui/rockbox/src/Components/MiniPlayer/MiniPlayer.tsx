import { FC } from "react";
import { useAtom } from "jotai";
import { controlBarState } from "../ControlBar/ControlBarState";
import { mobilePlayerState } from "../MobilePlayer/MobilePlayerState";
import Play from "../Icons/Play";
import Pause from "../Icons/Pause";
import Next from "../Icons/Next";
import Track from "../Icons/Track";
import {
  useNextMutation,
  usePauseMutation,
  useResumeMutation,
} from "../../Hooks/GraphQL";

const MiniPlayer: FC = () => {
  const [{ nowPlaying }] = useAtom(controlBarState);
  const [, setMobilePlayer] = useAtom(mobilePlayerState);
  const { mutate: pause } = usePauseMutation();
  const { mutate: resume } = useResumeMutation();
  const { mutate: next } = useNextMutation();

  if (!nowPlaying || nowPlaying.duration === 0) {
    return null;
  }

  const progress =
    nowPlaying.duration > 0
      ? (nowPlaying.progress / nowPlaying.duration) * 100
      : 0;

  const openPlayer = () => setMobilePlayer({ isOpen: true });

  return (
    <div className="md:hidden fixed bottom-[60px] left-0 right-0 z-40">
      <div
        className="mx-3 mb-2 rounded-[12px] overflow-hidden bg-[var(--theme-surface)] border border-[var(--theme-separator)] shadow-lg cursor-pointer active:opacity-90"
        onClick={openPlayer}
      >
        <div className="relative w-full h-[0.5px] bg-[rgba(177,178,181,0.15)]">
          <div
            className="absolute top-0 left-0 h-full bg-[#6F00FF]"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="flex flex-row items-center px-3 py-2 gap-3">
          {nowPlaying.cover ? (
            <img
              src={nowPlaying.cover}
              alt="cover"
              className="w-10 h-10 rounded-[6px] object-cover flex-shrink-0"
            />
          ) : (
            <div className="w-10 h-10 rounded-[6px] bg-[var(--theme-cover)] flex items-center justify-center flex-shrink-0">
              <Track color="#b1b2b5" height={18} width={18} />
            </div>
          )}

          <div className="flex flex-col flex-1 min-w-0">
            <div className="text-[13px] font-medium text-[var(--theme-text)] truncate">
              {nowPlaying.title}
            </div>
            <div className="text-[11px] text-[var(--theme-secondary-text)] truncate">
              {nowPlaying.artist}
            </div>
          </div>

          <button
            className="bg-transparent border-0 cursor-pointer p-2 flex items-center justify-center rounded-full active:opacity-60"
            onClick={(e) => {
              e.stopPropagation();
              nowPlaying.isPlaying ? pause({}) : resume({});
            }}
          >
            {nowPlaying.isPlaying ? (
              <Pause color="var(--theme-text)" small />
            ) : (
              <Play color="var(--theme-text)" small />
            )}
          </button>

          <button
            className="bg-transparent border-0 cursor-pointer p-2 flex items-center justify-center rounded-full active:opacity-60"
            onClick={(e) => {
              e.stopPropagation();
              next({});
            }}
          >
            <Next color="var(--theme-icon)" />
          </button>
        </div>
      </div>
    </div>
  );
};

export default MiniPlayer;

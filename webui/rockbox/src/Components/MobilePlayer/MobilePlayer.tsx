import { FC, useRef, useState } from "react";
import { useAtom } from "jotai";
import { controlBarState } from "../ControlBar/ControlBarState";
import { mobilePlayerState } from "./MobilePlayerState";
import { likesState } from "../Likes/LikesState";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Pause from "../Icons/Pause";
import Next from "../Icons/Next";
import Previous from "../Icons/Previous";
import Shuffle from "../Icons/Shuffle";
import Repeat from "../Icons/Repeat";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";
import Track from "../Icons/Track";
import { useTimeFormat } from "../../Hooks/useFormat";
import {
  useNextMutation,
  usePauseMutation,
  usePreviousMutation,
  useResumeMutation,
  useSeekMutation,
  useLikeTrackMutation,
  useUnlikeTrackMutation,
  useGetGlobalSettingsQuery,
  useSaveSettingsMutation,
  useGetBluetoothDevicesQuery,
} from "../../Hooks/GraphQL";
import { settingsState } from "../Settings/SettingsState";
import DeviceListWithData from "../ControlBar/DeviceList/DeviceListWithData";
import BluetoothListWithData from "../ControlBar/BluetoothList/BluetoothListWithData";
import PlayQueueWithData from "../ControlBar/PlayQueue/PlayQueueWithData";
import { List } from "@styled-icons/entypo";
import { Speaker } from "@styled-icons/bootstrap";

type Sheet = "queue" | "devices" | "bluetooth" | null;

const BluetoothIcon: FC<{ color: string }> = ({ color }) => (
  <svg viewBox="0 0 24 24" width={20} height={20} fill={color}>
    <path d="M17.71 7.71L12 2h-1v7.59L6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 11 14.41V22h1l5.71-5.71-4.3-4.29 4.3-4.29zM13 5.83l1.88 1.88L13 9.59V5.83zm1.88 10.46L13 18.17v-3.76l1.88 1.88z" />
  </svg>
);

const MobilePlayer: FC = () => {
  const [{ isOpen }, setMobilePlayer] = useAtom(mobilePlayerState);
  const [{ nowPlaying }] = useAtom(controlBarState);
  const [likes, setLikes] = useAtom(likesState);
  const [settings] = useAtom(settingsState);
  const { formatTime } = useTimeFormat();
  const progressRef = useRef<HTMLDivElement>(null);
  const [sheet, setSheet] = useState<Sheet>(null);

  const { mutate: pause } = usePauseMutation();
  const { mutate: resume } = useResumeMutation();
  const { mutate: next } = useNextMutation();
  const { mutate: previous } = usePreviousMutation();
  const { mutate: seek } = useSeekMutation();
  const { mutateAsync: likeTrack } = useLikeTrackMutation();
  const { mutateAsync: unlikeTrack } = useUnlikeTrackMutation();
  const { mutateAsync: saveSettings } = useSaveSettingsMutation();
  const { refetch: refetchSettings } = useGetGlobalSettingsQuery();
  const { isError: bluetoothUnavailable } = useGetBluetoothDevicesQuery({ retry: false });
  const bluetoothAvailable = !bluetoothUnavailable;

  if (!isOpen) return null;

  const close = () => {
    setSheet(null);
    setMobilePlayer({ isOpen: false });
  };

  const progress =
    nowPlaying && nowPlaying.duration > 0
      ? (nowPlaying.progress / nowPlaying.duration) * 100
      : 0;

  const handleProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!progressRef.current || !nowPlaying || nowPlaying.duration === 0) return;
    const rect = progressRef.current.getBoundingClientRect();
    const x = Math.max(0, e.clientX - rect.left);
    const pct = x / rect.width;
    seek({ elapsed: Math.floor(pct * nowPlaying.duration), offset: 0 });
  };

  const onLike = async () => {
    if (!nowPlaying?.id) return;
    setLikes((s) => ({ ...s, [nowPlaying.id!]: true }));
    await likeTrack({ trackId: nowPlaying.id });
  };

  const onUnlike = async () => {
    if (!nowPlaying?.id) return;
    setLikes((s) => ({ ...s, [nowPlaying.id!]: false }));
    await unlikeTrack({ trackId: nowPlaying.id });
  };

  const onShuffle = async () => {
    await saveSettings({ settings: { playlistShuffle: !settings.playlistShuffle } });
    await refetchSettings();
  };

  const onRepeat = async () => {
    await saveSettings({ settings: { repeatMode: settings.repeatMode === 0 ? 1 : 0 } });
    await refetchSettings();
  };

  const isLiked = likes[nowPlaying?.id || ""];
  const isShuffle = settings.playlistShuffle;
  const isRepeat = settings.repeatMode !== 0;

  return (
    <div className="md:hidden fixed inset-0 z-50 flex flex-col bg-[var(--theme-background)]">
      {nowPlaying?.cover && (
        <div
          className="absolute inset-0 bg-cover bg-center opacity-20 blur-[40px] scale-110 pointer-events-none"
          style={{ backgroundImage: `url(${nowPlaying.cover})` }}
        />
      )}

      <div className="relative flex flex-col h-full px-6">
        {/* Header */}
        <div className="flex items-center justify-between pt-10 pb-2">
          <button
            onClick={close}
            className="bg-transparent border-0 cursor-pointer p-2 -ml-2 flex items-center"
          >
            <ArrowBack color="var(--theme-icon)" />
          </button>
          <div className="text-[11px] uppercase tracking-widest text-[var(--theme-secondary-text)] font-medium">
            Now Playing
          </div>
          <div className="w-9" />
        </div>

        {/* Main player content */}
        <div className="flex-1 flex flex-col justify-center gap-6 min-h-0">
          {/* Album art */}
          <div className="flex items-center justify-center">
            {nowPlaying?.cover ? (
              <img
                src={nowPlaying.cover}
                alt="cover"
                className="w-56 h-56 rounded-[16px] object-cover shadow-2xl"
              />
            ) : (
              <div className="w-56 h-56 rounded-[16px] bg-[var(--theme-cover)] flex items-center justify-center shadow-2xl">
                <Track color="#b1b2b5" height={64} width={64} />
              </div>
            )}
          </div>

          {/* Track info + like */}
          <div className="flex flex-row items-center justify-between">
            <div className="flex flex-col flex-1 min-w-0">
              <div className="text-[20px] font-[RockfordSansMedium] text-[var(--theme-text)] truncate">
                {nowPlaying?.title || "—"}
              </div>
              <div className="text-[14px] text-[var(--theme-secondary-text)] truncate mt-[2px]">
                {nowPlaying?.artist || ""}
              </div>
            </div>
            <button
              className="bg-transparent border-0 cursor-pointer p-2 ml-4 flex-shrink-0"
              onClick={isLiked ? onUnlike : onLike}
            >
              {isLiked ? (
                <Heart color="#6F00FF" size={24} />
              ) : (
                <HeartOutline color="var(--theme-icon)" height={24} width={24} />
              )}
            </button>
          </div>

          {/* Progress bar */}
          <div className="flex flex-col gap-2">
            <div
              ref={progressRef}
              onClick={handleProgressClick}
              className="relative h-1 rounded-full bg-[rgba(177,178,181,0.25)] cursor-pointer"
            >
              <div
                className="absolute top-0 left-0 h-full rounded-full bg-[#6F00FF]"
                style={{ width: `${progress}%` }}
              />
              <div
                className="absolute top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-[#6F00FF] shadow-md"
                style={{ left: `calc(${progress}% - 6px)` }}
              />
            </div>
            <div className="flex flex-row justify-between text-[11px] text-[var(--theme-secondary-text)]">
              <span>{formatTime(nowPlaying?.progress ?? 0)}</span>
              <span>{formatTime(nowPlaying?.duration ?? 0)}</span>
            </div>
          </div>

          {/* Playback controls */}
          <div className="flex flex-row items-center justify-between px-2">
            <button
              onClick={onShuffle}
              className={`bg-transparent border-0 cursor-pointer p-3 rounded-full ${isShuffle ? "bg-[var(--theme-hover)]" : ""}`}
            >
              <Shuffle color={isShuffle ? "#6F00FF" : "var(--theme-icon)"} />
            </button>
            <button
              onClick={() => previous({})}
              className="bg-transparent border-0 cursor-pointer p-3 rounded-full"
            >
              <Previous color="var(--theme-icon)" />
            </button>
            <button
              onClick={() => (nowPlaying?.isPlaying ? pause({}) : resume({}))}
              className="bg-[#6F00FF] border-0 cursor-pointer w-16 h-16 rounded-full flex items-center justify-center shadow-lg"
            >
              {nowPlaying?.isPlaying ? (
                <Pause color="#fff" />
              ) : (
                <Play color="#fff" small />
              )}
            </button>
            <button
              onClick={() => next({})}
              className="bg-transparent border-0 cursor-pointer p-3 rounded-full"
            >
              <Next color="var(--theme-icon)" />
            </button>
            <button
              onClick={onRepeat}
              className={`bg-transparent border-0 cursor-pointer p-3 rounded-full ${isRepeat ? "bg-[var(--theme-hover)]" : ""}`}
            >
              <Repeat color={isRepeat ? "#6F00FF" : "var(--theme-icon)"} />
            </button>
          </div>

          {/* Action bar: Queue / Devices / Bluetooth */}
          <div className="flex flex-row items-center justify-around border-t border-[var(--theme-separator)] pt-4 pb-2">
            <button
              onClick={() => setSheet(sheet === "queue" ? null : "queue")}
              className={`flex flex-col items-center gap-1 bg-transparent border-0 cursor-pointer p-2 rounded-xl ${sheet === "queue" ? "bg-[var(--theme-hover)]" : ""}`}
            >
              <List size={22} color={sheet === "queue" ? "#6F00FF" : "var(--theme-icon)"} />
              <span className="text-[10px]" style={{ color: sheet === "queue" ? "#6F00FF" : "var(--theme-secondary-text)" }}>Queue</span>
            </button>
            <button
              onClick={() => setSheet(sheet === "devices" ? null : "devices")}
              className={`flex flex-col items-center gap-1 bg-transparent border-0 cursor-pointer p-2 rounded-xl ${sheet === "devices" ? "bg-[var(--theme-hover)]" : ""}`}
            >
              <Speaker size={22} color={sheet === "devices" ? "#6F00FF" : "var(--theme-icon)"} />
              <span className="text-[10px]" style={{ color: sheet === "devices" ? "#6F00FF" : "var(--theme-secondary-text)" }}>Devices</span>
            </button>
            {bluetoothAvailable && (
              <button
                onClick={() => setSheet(sheet === "bluetooth" ? null : "bluetooth")}
                className={`flex flex-col items-center gap-1 bg-transparent border-0 cursor-pointer p-2 rounded-xl ${sheet === "bluetooth" ? "bg-[var(--theme-hover)]" : ""}`}
              >
                <BluetoothIcon color={sheet === "bluetooth" ? "#6F00FF" : "var(--theme-icon)"} />
                <span className="text-[10px]" style={{ color: sheet === "bluetooth" ? "#6F00FF" : "var(--theme-secondary-text)" }}>Bluetooth</span>
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Bottom sheet overlay */}
      {sheet && (
        <div className="absolute inset-0 z-10 flex flex-col justify-end" onClick={() => setSheet(null)}>
          <div
            className="bg-[var(--theme-surface)] rounded-t-[20px] max-h-[70vh] flex flex-col overflow-hidden"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Drag handle */}
            <div className="flex justify-center pt-3 pb-2 flex-shrink-0">
              <div className="w-10 h-[4px] rounded-full bg-[var(--theme-separator)]" />
            </div>

            {/* Sheet title + close */}
            <div className="flex items-center justify-between px-5 pb-3 flex-shrink-0">
              <div className="text-[15px] font-[RockfordSansMedium] text-[var(--theme-text)]">
                {sheet === "queue" && "Play Queue"}
                {sheet === "devices" && "Output Devices"}
                {sheet === "bluetooth" && "Bluetooth"}
              </div>
              <button
                onClick={() => setSheet(null)}
                className="bg-transparent border-0 cursor-pointer text-[var(--theme-secondary-text)] text-[13px] px-2 py-1"
              >
                Done
              </button>
            </div>

            {/* Sheet content — full-width, scrollable */}
            <div className="flex-1 overflow-y-auto min-h-0">
              {sheet === "queue" && (
                <MobilePlayQueue />
              )}
              {sheet === "devices" && (
                <DeviceListWithData close={() => setSheet(null)} />
              )}
              {sheet === "bluetooth" && (
                <BluetoothListWithData close={() => setSheet(null)} />
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

const MobilePlayQueue: FC = () => (
  <div className="w-full">
    <PlayQueueWithData />
  </div>
);

export default MobilePlayer;

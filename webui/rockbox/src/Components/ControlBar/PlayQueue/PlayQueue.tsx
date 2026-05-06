import { FC, useMemo, useRef, useState } from "react";
import { Track } from "../../../Types/track";
import { Link } from "react-router-dom";
import TrackIcon from "../../Icons/Track";
import { CloseOutline } from "@styled-icons/evaicons-outline";
import { Play } from "@styled-icons/ionicons-sharp";
import { LazyLoadImage } from "react-lazy-load-image-component";
import "./styles.css";
import { useVirtualizer } from "@tanstack/react-virtual";

export type PlayQueueProps = {
  previousTracks?: Track[];
  nextTracks?: Track[];
  currentTrack?: Track;
  onPlayTrackAt: (index: number) => void;
  onRemoveTrackAt: (index: number) => void;
};

const PlayQueue: FC<PlayQueueProps> = ({
  previousTracks = [],
  nextTracks = [],
  onPlayTrackAt,
  onRemoveTrackAt,
}) => {
  const [active, setActive] = useState("playqueue");
  const parentRef = useRef<HTMLDivElement>(null);
  const { currentIndex, amount } = useMemo(() => {
    return {
      currentIndex: previousTracks.length,
      amount: previousTracks.length + nextTracks.length,
    };
  }, [previousTracks.length, nextTracks.length]);

  // The virtualizer
  const rowVirtualizer = useVirtualizer({
    count: active === "playqueue" ? nextTracks.length : previousTracks.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 64,
  });

  const onSwitch = () => {
    if (active === "playqueue") {
      setActive("history");
      return;
    }
    setActive("playqueue");
  };

  const _onPlayTrackAt = (index: number) => {
    if (active === "playqueue") {
      onPlayTrackAt((previousTracks?.length || 0) + index);
      return;
    }
    onPlayTrackAt(index);
  };

  const _onRemoveTrack = (index: number) => {
    if (active === "playqueue") {
      onRemoveTrackAt((previousTracks?.length || 0) + index);
      return;
    }
    onRemoveTrackAt(index);
  };

  const tracks = active === "playqueue" ? nextTracks! : previousTracks!;
  return (
    <div className="h-[calc(100vh-113px)] w-[370px]">
      <div className="flex flex-row">
        <div className="text-sm ml-4 mr-4 pt-5 mb-5 flex-1">
          {active === "playqueue" ? "Play Queue" : "History"}
        </div>
        {amount > 0 && (
          <div
            className="text-sm ml-4 mr-4 pt-5 mb-5 flex-1"
            style={{ fontSize: 14, color: "#616161", textAlign: "center" }}
          >
            {currentIndex}
            {"  /  "}
            {amount}
          </div>
        )}
        <div
          className="text-sm ml-4 mr-4 pt-5 mb-5 flex-1 text-primary text-right cursor-pointer select-none"
          onClick={onSwitch}
        >
          {active === "playqueue" ? "History" : "Play Queue"}
        </div>
      </div>
      <div ref={parentRef} className="h-[calc(100%-59.5px)] overflow-y-auto overflow-x-hidden">
        <div
          style={{
            height: rowVirtualizer.getTotalSize()
              ? `${rowVirtualizer.getTotalSize()}px`
              : undefined,
            width: "100%",
            position: "relative",
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualItem) => (
            <div
              key={virtualItem.key}
              className="flex flex-row h-16 items-center pl-4 cursor-pointer hover:bg-[var(--theme-hover)]"
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "calc(100% - 12px)",
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              {tracks[virtualItem.index].cover && (
                <div className="album-cover-container queue">
                  <LazyLoadImage
                    src={tracks[virtualItem.index].cover!}
                    className="h-12 w-12 rounded-[4px] mr-[18px] cursor-pointer"
                  />
                  <div
                    onClick={() => _onPlayTrackAt(virtualItem.index)}
                    className="floating-play queue"
                  >
                    <Play
                      size={16}
                      color={tracks[virtualItem.index].cover ? "#fff" : "#000"}
                    />
                  </div>
                </div>
              )}
              {!tracks[virtualItem.index].cover && (
                <div className="album-cover-container">
                  <div className="h-12 w-12 rounded-[4px] cursor-pointer bg-[var(--theme-cover)] flex justify-center items-center">
                    <TrackIcon width={28} height={28} color="#a4a3a3" />
                  </div>
                  <div
                    onClick={() => _onPlayTrackAt(virtualItem.index)}
                    className="floating-play queue"
                    style={{ left: 19, top: 10 }}
                  >
                    <Play
                      size={16}
                      color={tracks[virtualItem.index].cover ? "#fff" : "#000"}
                    />
                  </div>
                </div>
              )}
              <div className="flex min-w-[222px] flex-col flex-1">
                <div className="text-sm font-[RockfordSansMedium] text-ellipsis overflow-hidden whitespace-nowrap">
                  {tracks[virtualItem.index].title}
                </div>
                <Link
                  to={`/artists/${tracks[virtualItem.index].artistId}`}
                  style={{ textDecoration: "none" }}
                >
                  <div className="text-sm text-[var(--theme-secondary-text)] text-ellipsis overflow-hidden whitespace-nowrap">
                    {tracks[virtualItem.index].artist}
                  </div>
                </Link>
              </div>
              <button
                className="bg-transparent cursor-pointer border-0 mr-[10px]"
                onClick={() => _onRemoveTrack(virtualItem.index)}
              >
                <CloseOutline size={24} color="var(--theme-text)" />
              </button>
            </div>
          ))}
          {tracks.length === 0 && active === "playqueue" && (
            <div className="flex items-center justify-center h-full text-center px-5 text-sm">
              No upcoming tracks. Add some to your play queue.
            </div>
          )}
          {tracks.length === 0 && active === "history" && (
            <div className="flex items-center justify-center h-full text-center px-5 text-sm">
              No history. Play some tracks to see them here.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PlayQueue;

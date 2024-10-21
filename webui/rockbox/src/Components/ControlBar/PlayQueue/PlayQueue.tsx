import { FC, useMemo, useRef, useState } from "react";
import { Track } from "../../../Types/track";
import { Link } from "react-router-dom";
import { useTheme } from "@emotion/react";
import TrackIcon from "../../Icons/Track";
import { CloseOutline } from "@styled-icons/evaicons-outline";
import { Play } from "@styled-icons/ionicons-sharp";
import {
  AlbumCover,
  AlbumCoverAlt,
  Artist,
  Container,
  Header,
  List,
  ListItem,
  Placeholder,
  Remove,
  Switch,
  Title,
  TrackDetails,
  TrackTitle,
} from "./styles";
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
  const theme = useTheme();
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
    <Container>
      <Header>
        <Title>{active === "playqueue" ? "Play Queue" : "History"}</Title>
        {amount > 0 && (
          <Title
            style={{ fontSize: 14, color: "#616161", textAlign: "center" }}
          >
            {currentIndex}
            {"  /  "}
            {amount}
          </Title>
        )}
        <Switch onClick={onSwitch}>
          {active === "playqueue" ? "History" : "Play Queue"}
        </Switch>
      </Header>
      <List ref={parentRef}>
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
            <ListItem
              key={virtualItem.key}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "calc(100% - 34px)",
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              {tracks[virtualItem.index].cover && (
                <div className="album-cover-container queue">
                  <AlbumCover src={tracks[virtualItem.index].cover!} />
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
                  <AlbumCoverAlt>
                    <TrackIcon width={28} height={28} color="#a4a3a3" />
                  </AlbumCoverAlt>
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
              <TrackDetails>
                <TrackTitle>{tracks[virtualItem.index].title}</TrackTitle>
                <Link
                  to={`/artists/${tracks[virtualItem.index].artistId}`}
                  style={{ textDecoration: "none" }}
                >
                  <Artist>{tracks[virtualItem.index].artist}</Artist>
                </Link>
              </TrackDetails>
              <Remove onClick={() => _onRemoveTrack(virtualItem.index)}>
                <CloseOutline size={24} color={theme.colors.text} />
              </Remove>
            </ListItem>
          ))}
          {tracks.length === 0 && active === "playqueue" && (
            <Placeholder>
              No upcoming tracks. Add some to your play queue.
            </Placeholder>
          )}
          {tracks.length === 0 && active === "history" && (
            <Placeholder>
              No history. Play some tracks to see them here.
            </Placeholder>
          )}
        </div>
      </List>
    </Container>
  );
};

export default PlayQueue;

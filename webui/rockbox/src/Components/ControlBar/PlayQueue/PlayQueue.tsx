import { FC, useState } from "react";
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
        <Switch onClick={onSwitch}>
          {active === "playqueue" ? "History" : "Play Queue"}
        </Switch>
      </Header>
      <List>
        {tracks.map((track, index) => (
          <ListItem key={track.id}>
            {track.cover && (
              <div className="album-cover-container">
                <AlbumCover src={track.cover} />
                <div
                  onClick={() => _onPlayTrackAt(index)}
                  className="floating-play"
                >
                  <Play size={16} color={track.cover ? "#fff" : "#000"} />
                </div>
              </div>
            )}
            {!track.cover && (
              <div className="album-cover-container">
                <AlbumCoverAlt>
                  <TrackIcon width={28} height={28} color="#a4a3a3" />
                </AlbumCoverAlt>
                <div
                  onClick={() => _onPlayTrackAt(index)}
                  className="floating-play"
                >
                  <Play size={16} color={track.cover ? "#fff" : "#000"} />
                </div>
              </div>
            )}
            <TrackDetails>
              <TrackTitle>{track.title}</TrackTitle>
              <Link
                to={`/artists/${track.artistId}`}
                style={{ textDecoration: "none" }}
              >
                <Artist>{track.artist}</Artist>
              </Link>
            </TrackDetails>
            <Remove onClick={() => _onRemoveTrack(index)}>
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
      </List>
    </Container>
  );
};

export default PlayQueue;

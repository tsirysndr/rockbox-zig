import { FC } from "react";
import Play from "../Icons/Play";
import Previous from "../Icons/Previous";
import Next from "../Icons/Next";
import Shuffle from "../Icons/Shuffle";
import Repeat from "../Icons/Repeat";
import { Button, Container, Controls, ControlsContainer } from "./styles";
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
};

const ControlBar: FC<ControlBarProps> = (props) => {
  return (
    <Container>
      <Controls>
        <ControlsContainer>
          <Button onClick={props.onShuffle}>
            <Shuffle />
          </Button>
          <Button onClick={props.onPrevious}>
            <Previous />
          </Button>
          {!props.nowPlaying?.isPlaying && (
            <Button onClick={props.onPlay}>
              <Play />
            </Button>
          )}
          {props.nowPlaying?.isPlaying && (
            <Button onClick={props.onPause}>
              <Pause />
            </Button>
          )}
          <Button onClick={props.onNext}>
            <Next />
          </Button>
          <Button onClick={props.onRepeat}>
            <Repeat />
          </Button>
        </ControlsContainer>
      </Controls>
      <CurrentTrack
        nowPlaying={props.nowPlaying}
        liked={props.liked}
        onLike={props.onLike}
        onUnlike={props.onUnlike}
      />
      <RightMenu />
    </Container>
  );
};

export default ControlBar;

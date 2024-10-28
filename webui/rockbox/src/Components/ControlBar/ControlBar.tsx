import { FC, useEffect, useState } from "react";
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
    <Container>
      <Controls>
        <ControlsContainer>
          <Button onClick={onShuffle} active={shuffle}>
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
          <Button onClick={onRepeat} active={repeat}>
            <Repeat />
          </Button>
        </ControlsContainer>
      </Controls>
      <CurrentTrack
        nowPlaying={props.nowPlaying}
        liked={props.liked}
        onLike={props.onLike}
        onUnlike={props.onUnlike}
        onSeek={props.onSeek}
      />
      <RightMenu />
    </Container>
  );
};

export default ControlBar;

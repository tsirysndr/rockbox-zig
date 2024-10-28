import { FC, useState, useEffect } from "react";
import styles, { Item, Section, SettingsTitle } from "./styles";
import Switch from "../../Switch";
import { Slider } from "@mui/material";
import { Select } from "baseui/select";
import {
  crossfadeOptions,
  crossfadeValues,
  fadeOutModeOptions,
  fadeOutModeValues,
  repeatOptions,
  repeatValues,
  replaygainOptions,
  replaygainValues,
} from "./consts";

export type PlaybackProps = {
  shuffle: boolean;
  repeat: number;
  fadeOnStopPause: boolean;
  crossfade: number;
  replaygain: number;
  fadeInDelay: number;
  fadeInDuration: number;
  fadeOutDelay: number;
  fadeOutDuration: number;
  fadeOutMode: number;
  onShuffleChange: (shuffle: boolean) => void;
  onRepeatChange: (repeat: number) => void;
  onFadeOnStopPauseChange: (fadeOnStopPause: boolean) => void;
  onCrossfadeChange: (crossfade: number) => void;
  onReplaygainChange: (replaygain: number) => void;
  onFadeInDelayChange: (fadeInDelay: number) => void;
  onFadeInDurationChange: (fadeInDuration: number) => void;
  onFadeOutDelayChange: (fadeOutDelay: number) => void;
  onFadeOutDurationChange: (fadeOutDuration: number) => void;
  onFadeOutModeChange: (fadeOutMode: number) => void;
};

const Playback: FC<PlaybackProps> = (props) => {
  const [shuffle, setShuffle] = useState(props.shuffle);
  const [repeat, setRepeat] = useState(repeatValues[props.repeat]);
  const [fadeOnStopPause, setFadeOnStopPause] = useState(props.fadeOnStopPause);
  const [crossfade, setCrossfade] = useState(crossfadeValues[props.crossfade]);
  const [replaygain, setReplaygain] = useState(
    replaygainValues[props.replaygain]
  );
  const [fadeInDelay, setFadeInDelay] = useState(props.fadeInDelay);
  const [fadeInDuration, setFadeInDuration] = useState(props.fadeInDuration);
  const [fadeOutDelay, setFadeOutDelay] = useState(props.fadeOutDelay);
  const [fadeOutDuration, setFadeOutDuration] = useState(props.fadeOutDuration);
  const [fadeOutMode, setFadeOutMode] = useState(
    fadeOutModeValues[props.fadeOutMode]
  );

  useEffect(() => {
    setShuffle(props.shuffle);
    setRepeat(repeatValues[props.repeat]);
    setFadeOnStopPause(props.fadeOnStopPause);
    setCrossfade(crossfadeValues[props.crossfade]);
    setReplaygain(replaygainValues[props.replaygain]);
    setFadeInDelay(props.fadeInDelay); // 0 - 7 s
    setFadeInDuration(props.fadeInDuration); // 0 - 15 s
    setFadeOutDelay(props.fadeOutDelay); // 0 - 7 s
    setFadeOutDuration(props.fadeOutDuration); // 0 - 15 s
    setFadeOutMode(fadeOutModeValues[props.fadeOutMode]); // crossfade | mix
  }, [
    props.shuffle,
    props.repeat,
    props.fadeOnStopPause,
    props.crossfade,
    props.replaygain,
    props.fadeInDelay,
    props.fadeInDuration,
    props.fadeOutDelay,
    props.fadeOutDuration,
    props.fadeOutMode,
  ]);

  const onShuffleChange = (shuffle: boolean) => {
    setShuffle(shuffle);
    props.onShuffleChange(shuffle);
  };

  const onRepeatChange = (repeat: number) => {
    setRepeat(repeatValues[repeat]);
    props.onRepeatChange(repeat);
  };

  const onFadeOnStopPauseChange = (fadeOnStopPause: boolean) => {
    setFadeOnStopPause(fadeOnStopPause);
    props.onFadeOnStopPauseChange(fadeOnStopPause);
  };

  const onCrossfadeChange = (crossfade: number) => {
    setCrossfade(crossfadeValues[crossfade]);
    props.onCrossfadeChange(crossfade);
  };

  const onReplaygainChange = (replaygain: number) => {
    setReplaygain(replaygainValues[replaygain]);
    props.onReplaygainChange(replaygain);
  };

  const onFadeInDelayChange = (fadeInDelay: number) => {
    setFadeInDelay(fadeInDelay);
    props.onFadeInDelayChange(fadeInDelay);
  };

  const onFadeInDurationChange = (fadeInDuration: number) => {
    setFadeInDuration(fadeInDuration);
    props.onFadeInDurationChange(fadeInDuration);
  };

  const onFadeOutDelayChange = (fadeOutDelay: number) => {
    setFadeOutDelay(fadeOutDelay);
    props.onFadeOutDelayChange(fadeOutDelay);
  };

  const onFadeOutDurationChange = (fadeOutDuration: number) => {
    setFadeOutDuration(fadeOutDuration);
    props.onFadeOutDurationChange(fadeOutDuration);
  };

  const onFadeOutModeChange = (fadeOutMode: number) => {
    setFadeOutMode(fadeOutModeValues[fadeOutMode]);
    props.onFadeOutModeChange(fadeOutMode);
  };

  return (
    <>
      <SettingsTitle>Playback</SettingsTitle>
      <Section>
        <Item>
          <div>Shuffle</div>
          <div>
            <Switch
              checked={shuffle}
              onChange={() => onShuffleChange(!shuffle)}
            />
          </div>
        </Item>
        <Item>
          <div>Repeat</div>
          <div style={{ width: 280 }}>
            <Select
              options={repeatOptions}
              value={repeat}
              onChange={(params) => {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                onRepeatChange((params.value as any)[0].value);
              }}
              clearable={false}
            />
          </div>
        </Item>

        <Item>
          <div>Fade on Stop/Pause</div>
          <div>
            <Switch
              checked={fadeOnStopPause}
              onChange={() => onFadeOnStopPauseChange(!fadeOnStopPause)}
            />
          </div>
        </Item>

        <Item>
          <div>Crossfade</div>
          <div style={{ width: 280 }}>
            <Select
              options={crossfadeOptions}
              value={crossfade}
              onChange={(params) => {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                onCrossfadeChange((params.value as any)[0].value);
              }}
              clearable={false}
            />
          </div>
        </Item>

        <Item>
          <div>Fade-In Delay</div>
          <div style={{ width: 120 }}>
            <Slider
              value={fadeInDelay}
              onChange={(_event, value) => setFadeInDelay(value as number)}
              onChangeCommitted={(_event, value) =>
                onFadeInDelayChange(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              min={0}
              max={7}
              step={1}
              valueLabelFormat={(value) => `${value} s`}
            />
          </div>
        </Item>

        <Item>
          <div>Fade-In Duration</div>
          <div style={{ width: 120 }}>
            <Slider
              value={fadeInDuration}
              onChange={(_event, value) => setFadeInDuration(value as number)}
              onChangeCommitted={(_event, value) =>
                onFadeInDurationChange(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              min={0}
              max={15}
              valueLabelFormat={(value) => `${value} s`}
            />
          </div>
        </Item>

        <Item>
          <div>Fade-Out Delay</div>
          <div style={{ width: 120 }}>
            <Slider
              value={fadeOutDelay}
              onChange={(_event, value) => setFadeOutDelay(value as number)}
              onChangeCommitted={(_event, value) =>
                onFadeOutDelayChange(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              min={0}
              max={7}
              step={1}
              valueLabelFormat={(value) => `${value} s`}
            />
          </div>
        </Item>
        <Item>
          <div>Fade-Out Duration</div>
          <div style={{ width: 120 }}>
            <Slider
              value={fadeOutDuration}
              onChange={(_event, value) => setFadeOutDuration(value as number)}
              onChangeCommitted={(_event, value) =>
                onFadeOutDurationChange(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              min={0}
              max={15}
              valueLabelFormat={(value) => `${value} s`}
            />
          </div>
        </Item>
        <Item>
          <div>Fade-Out Mode</div>
          <div style={{ width: 280 }}>
            <Select
              options={fadeOutModeOptions}
              value={fadeOutMode}
              onChange={(params) => {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                onFadeOutModeChange((params.value as any)[0].value);
              }}
              clearable={false}
            />
          </div>
        </Item>

        <Item>
          <div>Replaygain</div>
          <div style={{ width: 280 }}>
            <Select
              options={replaygainOptions}
              value={replaygain}
              onChange={(params) => {
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                onReplaygainChange((params.value as any)[0].value);
              }}
              clearable={false}
            />
          </div>
        </Item>
      </Section>
    </>
  );
};

export default Playback;

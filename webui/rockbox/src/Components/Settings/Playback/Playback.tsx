import { FC, useState, useEffect } from "react";
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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const iOSBoxShadow =
  "0 3px 1px rgba(0,0,0,0.1),0 4px 8px rgba(0,0,0,0.13),0 0 0 1px rgba(0,0,0,0.02)";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const styles = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  slider: (_t: any) => ({
    color: "#6F00FF",
    "& .MuiSlider-track": {
      border: "none",
    },
    "& .MuiSlider-thumb": {
      width: 18,
      height: 18,
      backgroundColor: "#fff",
      "&::before": {
        boxShadow: "0 4px 8px rgba(0,0,0,0.18)",
      },
      "&:hover, &.Mui-focusVisible, &.Mui-active": {
        boxShadow: "none",
      },
    },
  }),
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  sliderIOS: (theme: any) => ({
    color: "#6F00FF",

    "& .MuiSlider-thumb": {
      height: 20,
      width: 20,
      backgroundColor: "#fff",
      boxShadow: "0 0 2px 0px rgba(0, 0, 0, 0.1)",
      "&:focus, &:hover, &.Mui-active": {
        boxShadow: "0px 0px 3px 1px rgba(0, 0, 0, 0.1)",
        // Reset on touch devices, it doesn't add specificity
        "@media (hover: none)": {
          boxShadow: iOSBoxShadow,
        },
      },
      "&:before": {
        boxShadow:
          "0px 0px 1px 0px rgba(0,0,0,0.2), 0px 0px 0px 0px rgba(0,0,0,0.14), 0px 0px 1px 0px rgba(0,0,0,0.12)",
      },
    },
    "& .MuiSlider-valueLabel": {
      fontSize: 12,
      fontWeight: "normal",
      top: -6,
      backgroundColor: "unset",
      color: theme.palette.text.primary,
      "&::before": {
        display: "none",
      },
      "& *": {
        background: "transparent",
        color: "inherit",
      },
    },
    "& .MuiSlider-track": {
      border: "none",
      height: 5,
    },
    "& .MuiSlider-rail": {
      opacity: 0.5,
      boxShadow: "inset 0px 0px 4px -2px #000",
      backgroundColor: "#d0d0d0",
    },
    ...theme.applyStyles("dark", {
      color: "#6F00FF",
    }),
  }),
};

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
      <div className="text-base font-semibold mb-4">Playback</div>
      <div className="mb-[50px] text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between min-h-[80px]">
          <div>Shuffle</div>
          <div>
            <Switch
              checked={shuffle}
              onChange={() => onShuffleChange(!shuffle)}
            />
          </div>
        </div>
        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
          <div>Fade on Stop/Pause</div>
          <div>
            <Switch
              checked={fadeOnStopPause}
              onChange={() => onFadeOnStopPauseChange(!fadeOnStopPause)}
            />
          </div>
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>
        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>
        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>

        <div className="flex flex-row items-center justify-between min-h-[80px]">
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
        </div>
      </div>
    </>
  );
};

export default Playback;

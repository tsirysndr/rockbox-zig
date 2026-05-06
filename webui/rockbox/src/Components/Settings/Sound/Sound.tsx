import { FC, useEffect, useState } from "react";
import { Slider } from "@mui/material";
import Equalizer from "./Equalizer";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const iOSBoxShadow =
  "0 3px 1px rgba(0,0,0,0.1),0 4px 8px rgba(0,0,0,0.13),0 0 0 1px rgba(0,0,0,0.02)";

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

export type SoundProps = {
  bass: number;
  treble: number;
  balance: number;
  onBassChange: (bass: number) => void;
  onTrebleChange: (treble: number) => void;
  onBalanceChange: (balance: number) => void;
};

const Sound: FC<SoundProps> = (props) => {
  const [bass, setBass] = useState(props.bass);
  const [treble, setTreble] = useState(props.treble);
  const [balance, setBalance] = useState(props.balance);

  useEffect(() => {
    setBass(props.bass);
    setTreble(props.treble);
    setBalance(props.balance);
  }, [props.bass, props.treble, props.balance]);

  const computeCutOff = (cutoff: number) => {
    // -24 dB to 24 dB
    return ((cutoff + 24) / 48) * 100;
  };

  const handleBassChange = (value: number) => {
    setBass(Math.floor((value / 100) * 48 - 24));
  };

  const handleTrebleChange = (value: number) => {
    setTreble(Math.floor((value / 100) * 48 - 24));
  };

  const handleBalanceChange = (value: number) => {
    setBalance(value);
  };

  const onBassChangeCommitted = (value: number) => {
    props.onBassChange(Math.floor((value / 100) * 48 - 24));
  };

  const onTrebleChangeCommitted = (value: number) => {
    props.onTrebleChange(Math.floor((value / 100) * 48 - 24));
  };

  const onBalanceChangeCommitted = (value: number) => {
    props.onBalanceChange(value);
  };

  return (
    <>
      <div className="text-base font-semibold mb-4">Sound</div>
      <div className="mb-[50px] text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Bass</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(bass)}
              onChange={(_event, value) => handleBassChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onBassChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) =>
                `${Math.floor((value / 100) * 48 - 24)} dB`
              }
              min={0}
              max={100}
              step={1}
            />
          </div>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Treble</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(treble)}
              onChange={(_event, value) => handleTrebleChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onTrebleChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) =>
                `${Math.floor((value / 100) * 48 - 24)} dB`
              }
              min={0}
              max={100}
              step={1}
            />
          </div>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Balance</div>
          <div style={{ width: 120 }}>
            <Slider
              value={balance}
              onChange={(_event, value) => handleBalanceChange(value as number)}
              onChangeCommitted={(_event, value) =>
                onBalanceChangeCommitted(value as number)
              }
              sx={styles.slider}
              valueLabelDisplay="auto"
              valueLabelFormat={(value) => `${value} %`}
              min={-100}
              max={100}
              step={1}
            />
          </div>
        </div>
        <Equalizer />
      </div>
    </>
  );
};

export default Sound;

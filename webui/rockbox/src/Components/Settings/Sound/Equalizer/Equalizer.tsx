import { FC, useEffect, useState } from "react";
import Switch from "../../../Switch";
import { Slider } from "@mui/material";
import BigNumber from "bignumber.js";

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

export type EqualizerProps = {
  eqEnabled: boolean;
  onEnableEq: (enabled: boolean) => void;
  eqBandSettings: {
    q: number;
    gain: number;
    cutoff: number;
  }[];
  onEqBandSettingsChange: (
    bandSettings: {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  ) => void;
};

const Equalizer: FC<EqualizerProps> = (props) => {
  const [eqEnabled, setEqEnabled] = useState(props.eqEnabled);
  const [eqBandSettings, setEqBandSettings] = useState<
    {
      q: number;
      gain: number;
      cutoff: number;
    }[]
  >(props.eqBandSettings);

  useEffect(() => {
    setEqEnabled(props.eqEnabled);
    setEqBandSettings(props.eqBandSettings);
  }, [props.eqEnabled, props.eqBandSettings]);

  const formatLabel = (value: number) => {
    const labels: Record<number, string> = {
      64: "1 Hz",
      125: "64 Hz",
      250: "125 Hz",
      500: "250 Hz",
      1000: "500 Hz",
      2000: "1 kHz",
      4000: "2 kHz",
      8000: "4 kHz",
      16000: "8 kHz",
      0: "16 kHz",
    };
    return `${labels[value]}`;
  };

  const computeCutOff = (cutoff: number) => {
    // -24 dB to 24 dB
    return ((cutoff + 24) / 48) * 100;
  };

  const handleChange = (value: number, index: number) => {
    const newBandSettings = [...eqBandSettings];
    newBandSettings[index] = {
      ...newBandSettings[index],
      cutoff: Math.floor(((value / 100) * 48 - 24) * 10),
    };
    setEqBandSettings(newBandSettings);
  };

  const onChangeCommitted = (value: number, index: number) => {
    const newBandSettings = [...eqBandSettings];
    newBandSettings[index] = {
      ...newBandSettings[index],
      cutoff: Math.floor(((value / 100) * 48 - 24) * 10),
    };
    props.onEqBandSettingsChange(newBandSettings);
  };

  return (
    <>
      <div className="flex flex-row items-center justify-between h-[50px]">
        <div>Equalizer</div>
        <div>
          <Switch
            checked={eqEnabled}
            onChange={() => {
              props.onEnableEq(!eqEnabled);
              setEqEnabled(!eqEnabled);
            }}
          />
        </div>
      </div>
      <div>
        <div className="h-[250px] mx-auto mt-[50px] mb-[120px] flex flex-row justify-between w-[73%] text-[13px]">
          {eqBandSettings.map((band, index) => (
            <div key={index}>
              <Slider
                value={computeCutOff(band.cutoff * 0.1)}
                onChange={(_event, value) =>
                  handleChange(value as number, index)
                }
                onChangeCommitted={(_event, value) =>
                  onChangeCommitted(value as number, index)
                }
                sx={styles.sliderIOS}
                valueLabelDisplay="auto"
                orientation="vertical"
                min={0}
                max={100}
                step={0.1}
                valueLabelFormat={(value) =>
                  `${new BigNumber(value)
                    .dividedBy(100)
                    .multipliedBy(48)
                    .minus(24)
                    .toPrecision(2, 1)} dB`
                }
              />
              <div>{formatLabel(band.q)}</div>
            </div>
          ))}
        </div>
      </div>
    </>
  );
};

export default Equalizer;

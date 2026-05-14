import { FC, useEffect, useState } from "react";
import Switch from "../../../Switch";
import { Slider } from "@mui/material";

const iOSBoxShadow =
  "0 3px 1px rgba(0,0,0,0.1),0 4px 8px rgba(0,0,0,0.13),0 0 0 1px rgba(0,0,0,0.02)";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sliderSx = (theme: any) => ({
  color: "#6F00FF",
  "& .MuiSlider-thumb": {
    height: 20,
    width: 20,
    backgroundColor: "#fff",
    boxShadow: "0 0 2px 0px rgba(0,0,0,0.1)",
    "&:focus, &:hover, &.Mui-active": {
      boxShadow: "0px 0px 3px 1px rgba(0,0,0,0.1)",
      "@media (hover: none)": { boxShadow: iOSBoxShadow },
    },
    "&:before": {
      boxShadow:
        "0px 0px 1px 0px rgba(0,0,0,0.2), 0px 0px 0px 0px rgba(0,0,0,0.14), 0px 0px 1px 0px rgba(0,0,0,0.12)",
    },
  },
  "& .MuiSlider-valueLabel": {
    fontSize: 11,
    fontWeight: "normal",
    top: -6,
    backgroundColor: "unset",
    color: theme.palette.text.primary,
    "&::before": { display: "none" },
    "& *": { background: "transparent", color: "inherit" },
  },
  "& .MuiSlider-track": { border: "none", width: 5 },
  "& .MuiSlider-rail": {
    opacity: 0.5,
    boxShadow: "inset 0px 0px 4px -2px #000",
    backgroundColor: "#d0d0d0",
    width: 5,
  },
  ...theme.applyStyles("dark", { color: "#6F00FF" }),
});

/** Format a cutoff frequency in Hz as a human-readable label. */
function formatFreq(hz: number): string {
  if (hz >= 1000) return `${hz / 1000}kHz`;
  return `${hz}Hz`;
}

/** Format a gain value (tenths of dB) as a dB string. */
function formatGain(tenths: number): string {
  const db = (tenths / 10).toFixed(1);
  return tenths >= 0 ? `+${db}` : db;
}

export type EqualizerProps = {
  eqEnabled: boolean;
  onEnableEq: (enabled: boolean) => void;
  eqBandSettings: { q: number; gain: number; cutoff: number }[];
  onEqBandSettingsChange: (
    bandSettings: { q: number; gain: number; cutoff: number }[]
  ) => void;
};

const Equalizer: FC<EqualizerProps> = (props) => {
  const [eqEnabled, setEqEnabled] = useState(props.eqEnabled);
  const [bands, setBands] = useState(props.eqBandSettings);

  useEffect(() => {
    setEqEnabled(props.eqEnabled);
    setBands(props.eqBandSettings);
  }, [props.eqEnabled, props.eqBandSettings]);

  const handleGainChange = (value: number, index: number) => {
    const next = [...bands];
    next[index] = { ...next[index], gain: value };
    setBands(next);
  };

  const handleGainCommit = (value: number, index: number) => {
    const next = [...bands];
    next[index] = { ...next[index], gain: value };
    props.onEqBandSettingsChange(next);
  };

  return (
    <>
      <div className="flex flex-row items-center justify-between h-[50px]">
        <div>Equalizer</div>
        <Switch
          checked={eqEnabled}
          onChange={() => {
            props.onEnableEq(!eqEnabled);
            setEqEnabled(!eqEnabled);
          }}
        />
      </div>

      {/* Vertical band sliders */}
      <div className="mx-auto mt-[50px] mb-[120px] flex flex-row justify-between w-full text-[11px]">
        {bands.map((band, index) => (
          <div key={index} className="flex flex-col items-center gap-1" style={{ height: 250 }}>
            <div className="text-center text-[11px] text-[#ccc] font-mono leading-none mb-1">
              {formatGain(band.gain)}
            </div>
            <Slider
              value={band.gain}
              onChange={(_e, v) => handleGainChange(v as number, index)}
              onChangeCommitted={(_e, v) => handleGainCommit(v as number, index)}
              sx={sliderSx}
              valueLabelDisplay="off"
              orientation="vertical"
              min={-240}
              max={240}
              step={5}
              style={{ flex: 1 }}
            />
            <div className="text-center text-[#aaa] leading-tight mt-1">
              {formatFreq(band.cutoff)}
            </div>
          </div>
        ))}
      </div>
    </>
  );
};

export default Equalizer;

import { FC, useEffect, useState } from "react";
import { Slider } from "@mui/material";
import Switch from "../../../Switch";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sliderSx = (theme: any) => ({
  color: "#6F00FF",
  "& .MuiSlider-thumb": {
    width: 18,
    height: 18,
    backgroundColor: "#fff",
    "&::before": { boxShadow: "0 4px 8px rgba(0,0,0,0.18)" },
    "&:hover, &.Mui-focusVisible, &.Mui-active": { boxShadow: "none" },
  },
  "& .MuiSlider-track": { border: "none", height: 5 },
  "& .MuiSlider-rail": {
    opacity: 0.5,
    boxShadow: "inset 0px 0px 4px -2px #000",
    backgroundColor: "#d0d0d0",
  },
  ...theme.applyStyles("dark", { color: "#6F00FF" }),
});

const RG_MODES = ["Track", "Album", "Shuffle", "Off"];

export type ReplayGainProps = {
  type: number;
  preamp: number;
  noclip: boolean;
  onChange: (s: { type: number; preamp: number; noclip: boolean }) => void;
};

const ReplayGain: FC<ReplayGainProps> = (props) => {
  const [type, setType] = useState(props.type);
  const [preamp, setPreamp] = useState(props.preamp);
  const [noclip, setNoclip] = useState(props.noclip);

  useEffect(() => {
    setType(props.type);
    setPreamp(props.preamp);
    setNoclip(props.noclip);
  }, [props.type, props.preamp, props.noclip]);

  const commit = (overrides?: Partial<ReplayGainProps>) => {
    props.onChange({ type, preamp, noclip, ...overrides });
  };

  return (
    <div className="mt-4">
      <div className="text-base font-semibold mb-2">Replay Gain</div>
      <div className="text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Mode</div>
          <select
            value={type}
            onChange={(e) => {
              const v = Number(e.target.value);
              setType(v);
              commit({ type: v });
            }}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {RG_MODES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Pre-amp</div>
          <div className="flex items-center gap-2">
            <span className="text-[13px] text-[#aaa] w-14 text-right">
              {(preamp / 10).toFixed(1)} dB
            </span>
            <div style={{ width: 120 }}>
              <Slider
                value={preamp}
                onChange={(_e, v) => setPreamp(v as number)}
                onChangeCommitted={(_e, v) => { setPreamp(v as number); commit({ preamp: v as number }); }}
                sx={sliderSx}
                min={-120} max={120} step={5}
              />
            </div>
          </div>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Prevent clipping</div>
          <Switch
            checked={noclip}
            onChange={() => {
              const v = !noclip;
              setNoclip(v);
              commit({ noclip: v });
            }}
          />
        </div>
      </div>
    </div>
  );
};

export default ReplayGain;

import { FC, useEffect, useState } from "react";
import { Slider } from "@mui/material";

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

export type CrossfeedProps = {
  type: number;
  directGain: number;
  crossGain: number;
  hfAttenuation: number;
  hfCutoff: number;
  onChange: (settings: {
    type: number;
    directGain: number;
    crossGain: number;
    hfAttenuation: number;
    hfCutoff: number;
  }) => void;
};

const CROSSFEED_TYPES = ["Off", "Meier", "Custom"];

const Crossfeed: FC<CrossfeedProps> = (props) => {
  const [type, setType] = useState(props.type);
  const [directGain, setDirectGain] = useState(props.directGain);
  const [crossGain, setCrossGain] = useState(props.crossGain);
  const [hfAttenuation, setHfAttenuation] = useState(props.hfAttenuation);
  const [hfCutoff, setHfCutoff] = useState(props.hfCutoff);

  useEffect(() => {
    setType(props.type);
    setDirectGain(props.directGain);
    setCrossGain(props.crossGain);
    setHfAttenuation(props.hfAttenuation);
    setHfCutoff(props.hfCutoff);
  }, [props.type, props.directGain, props.crossGain, props.hfAttenuation, props.hfCutoff]);

  const commit = (overrides?: Partial<typeof props>) => {
    props.onChange({
      type,
      directGain,
      crossGain,
      hfAttenuation,
      hfCutoff,
      ...overrides,
    });
  };

  const isCustom = type === 2;

  return (
    <div className="mt-4">
      <div className="text-base font-semibold mb-2">Crossfeed</div>
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
            {CROSSFEED_TYPES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>

        {isCustom && (
          <>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Direct gain</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-14 text-right">
                  {(directGain / 10).toFixed(1)} dB
                </span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={directGain}
                    onChange={(_e, v) => setDirectGain(v as number)}
                    onChangeCommitted={(_e, v) => { setDirectGain(v as number); commit({ directGain: v as number }); }}
                    sx={sliderSx}
                    min={-600} max={0} step={5}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Cross gain</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-14 text-right">
                  {(crossGain / 10).toFixed(1)} dB
                </span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={crossGain}
                    onChange={(_e, v) => setCrossGain(v as number)}
                    onChangeCommitted={(_e, v) => { setCrossGain(v as number); commit({ crossGain: v as number }); }}
                    sx={sliderSx}
                    min={-1200} max={-300} step={10}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>HF attenuation</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-14 text-right">
                  {(hfAttenuation / 10).toFixed(1)} dB
                </span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={hfAttenuation}
                    onChange={(_e, v) => setHfAttenuation(v as number)}
                    onChangeCommitted={(_e, v) => { setHfAttenuation(v as number); commit({ hfAttenuation: v as number }); }}
                    sx={sliderSx}
                    min={-2400} max={-600} step={10}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>HF cutoff</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-14 text-right">
                  {hfCutoff} Hz
                </span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={hfCutoff}
                    onChange={(_e, v) => setHfCutoff(v as number)}
                    onChangeCommitted={(_e, v) => { setHfCutoff(v as number); commit({ hfCutoff: v as number }); }}
                    sx={sliderSx}
                    min={500} max={2000} step={50}
                  />
                </div>
              </div>
            </div>
          </>
        )}

        {type === 0 && (
          <div className="text-[13px] text-[#aaa] pb-3">
            Crossfeed is off. Meier mode blends stereo for headphone listening.
          </div>
        )}
      </div>
    </div>
  );
};

export default Crossfeed;

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

const CROSSFADE_MODES = [
  "Off",
  "Auto-skip",
  "Manual-skip",
  "Shuffle",
  "Shuffle + Manual",
  "Always",
];

export type CrossfadeProps = {
  mode: number;
  fadeOnStop: boolean;
  fadeInDelay: number;
  fadeOutDelay: number;
  fadeInDuration: number;
  fadeOutDuration: number;
  mixmode: number;
  onChange: (s: {
    mode: number;
    fadeOnStop: boolean;
    fadeInDelay: number;
    fadeOutDelay: number;
    fadeInDuration: number;
    fadeOutDuration: number;
    mixmode: number;
  }) => void;
};

const Crossfade: FC<CrossfadeProps> = (props) => {
  const [mode, setMode] = useState(props.mode);
  const [fadeOnStop, setFadeOnStop] = useState(props.fadeOnStop);
  const [fadeInDelay, setFadeInDelay] = useState(props.fadeInDelay);
  const [fadeOutDelay, setFadeOutDelay] = useState(props.fadeOutDelay);
  const [fadeInDuration, setFadeInDuration] = useState(props.fadeInDuration);
  const [fadeOutDuration, setFadeOutDuration] = useState(props.fadeOutDuration);
  const [mixmode, setMixmode] = useState(props.mixmode);

  useEffect(() => {
    setMode(props.mode);
    setFadeOnStop(props.fadeOnStop);
    setFadeInDelay(props.fadeInDelay);
    setFadeOutDelay(props.fadeOutDelay);
    setFadeInDuration(props.fadeInDuration);
    setFadeOutDuration(props.fadeOutDuration);
    setMixmode(props.mixmode);
  }, [props.mode, props.fadeOnStop, props.fadeInDelay, props.fadeOutDelay,
      props.fadeInDuration, props.fadeOutDuration, props.mixmode]);

  const commit = (overrides?: Partial<CrossfadeProps>) => {
    props.onChange({
      mode, fadeOnStop, fadeInDelay, fadeOutDelay,
      fadeInDuration, fadeOutDuration, mixmode, ...overrides,
    });
  };

  const isOn = mode !== 0;

  return (
    <div className="mt-4">
      <div className="text-base font-semibold mb-2">Crossfade</div>
      <div className="text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Mode</div>
          <select
            value={mode}
            onChange={(e) => {
              const v = Number(e.target.value);
              setMode(v);
              commit({ mode: v });
            }}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {CROSSFADE_MODES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>

        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Fade on stop</div>
          <Switch
            checked={fadeOnStop}
            onChange={() => {
              const v = !fadeOnStop;
              setFadeOnStop(v);
              commit({ fadeOnStop: v });
            }}
          />
        </div>

        {isOn && (
          <>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Fade-in delay</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-8 text-right">{fadeInDelay} s</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fadeInDelay}
                    onChange={(_e, v) => setFadeInDelay(v as number)}
                    onChangeCommitted={(_e, v) => { setFadeInDelay(v as number); commit({ fadeInDelay: v as number }); }}
                    sx={sliderSx}
                    min={0} max={7} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Fade-out delay</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-8 text-right">{fadeOutDelay} s</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fadeOutDelay}
                    onChange={(_e, v) => setFadeOutDelay(v as number)}
                    onChangeCommitted={(_e, v) => { setFadeOutDelay(v as number); commit({ fadeOutDelay: v as number }); }}
                    sx={sliderSx}
                    min={0} max={7} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Fade-in duration</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-8 text-right">{fadeInDuration} s</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fadeInDuration}
                    onChange={(_e, v) => setFadeInDuration(v as number)}
                    onChangeCommitted={(_e, v) => { setFadeInDuration(v as number); commit({ fadeInDuration: v as number }); }}
                    sx={sliderSx}
                    min={0} max={15} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Fade-out duration</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-8 text-right">{fadeOutDuration} s</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fadeOutDuration}
                    onChange={(_e, v) => setFadeOutDuration(v as number)}
                    onChangeCommitted={(_e, v) => { setFadeOutDuration(v as number); commit({ fadeOutDuration: v as number }); }}
                    sx={sliderSx}
                    min={0} max={15} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Fade-out type</div>
              <select
                value={mixmode}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  setMixmode(v);
                  commit({ mixmode: v });
                }}
                className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
              >
                <option value={0}>Crossfade</option>
                <option value={1}>Mix</option>
              </select>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default Crossfade;

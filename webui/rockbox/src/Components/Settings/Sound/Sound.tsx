import { FC, useEffect, useState } from "react";
import { Slider } from "@mui/material";
import Switch from "../../Switch";
import EqualizerWithData from "./Equalizer/EqualizerWithData";
import CrossfeedWithData from "./Crossfeed/CrossfeedWithData";
import SurroundWithData from "./Surround/SurroundWithData";
import CrossfadeWithData from "./Crossfade/CrossfadeWithData";
import ReplayGainWithData from "./ReplayGain/ReplayGainWithData";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sliderSx = (_t: any) => ({
  color: "#6F00FF",
  "& .MuiSlider-track": { border: "none" },
  "& .MuiSlider-thumb": {
    width: 18,
    height: 18,
    backgroundColor: "#fff",
    "&::before": { boxShadow: "0 4px 8px rgba(0,0,0,0.18)" },
    "&:hover, &.Mui-focusVisible, &.Mui-active": { boxShadow: "none" },
  },
});

const CHANNEL_MODES = ["Stereo", "Mono", "Custom", "Mono L", "Mono R", "Karaoke", "Swap"];
const AFR_MODES = ["Off", "Low", "Medium", "High"];
const PBE_MODES = ["Off", "Low", "Medium", "High"];

export type SoundProps = {
  bass: number;
  treble: number;
  balance: number;
  channelConfig: number;
  stereoWidth: number;
  ditheringEnabled: boolean;
  afrEnabled: number;
  pbe: number;
  pbePrecut: number;
  onBassChange: (v: number) => void;
  onTrebleChange: (v: number) => void;
  onBalanceChange: (v: number) => void;
  onChannelConfigChange: (v: number) => void;
  onStereoWidthChange: (v: number) => void;
  onDitheringChange: (v: boolean) => void;
  onAfrChange: (v: number) => void;
  onPbeChange: (pbe: number, precut: number) => void;
};

const Sound: FC<SoundProps> = (props) => {
  const [bass, setBass] = useState(props.bass);
  const [treble, setTreble] = useState(props.treble);
  const [balance, setBalance] = useState(props.balance);
  const [channelConfig, setChannelConfig] = useState(props.channelConfig);
  const [stereoWidth, setStereoWidth] = useState(props.stereoWidth);
  const [pbePrecut, setPbePrecut] = useState(props.pbePrecut);

  useEffect(() => {
    setBass(props.bass);
    setTreble(props.treble);
    setBalance(props.balance);
    setChannelConfig(props.channelConfig);
    setStereoWidth(props.stereoWidth);
    setPbePrecut(props.pbePrecut);
  }, [props.bass, props.treble, props.balance, props.channelConfig,
      props.stereoWidth, props.pbePrecut]);

  const computeCutOff = (v: number) => ((v + 24) / 48) * 100;
  const fromSlider = (v: number) => Math.floor((v / 100) * 48 - 24);

  return (
    <>
      <div className="text-base font-semibold mb-4">Sound</div>

      {/* ── Tone controls ──────────────────────────────────────────────── */}
      <div className="mb-4 text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Bass</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(bass)}
              onChange={(_e, v) => setBass(fromSlider(v as number))}
              onChangeCommitted={(_e, v) => props.onBassChange(fromSlider(v as number))}
              sx={sliderSx}
              valueLabelDisplay="auto"
              valueLabelFormat={(v) => `${fromSlider(v)} dB`}
              min={0} max={100} step={1}
            />
          </div>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Treble</div>
          <div style={{ width: 120 }}>
            <Slider
              value={computeCutOff(treble)}
              onChange={(_e, v) => setTreble(fromSlider(v as number))}
              onChangeCommitted={(_e, v) => props.onTrebleChange(fromSlider(v as number))}
              sx={sliderSx}
              valueLabelDisplay="auto"
              valueLabelFormat={(v) => `${fromSlider(v)} dB`}
              min={0} max={100} step={1}
            />
          </div>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Balance</div>
          <div style={{ width: 120 }}>
            <Slider
              value={balance}
              onChange={(_e, v) => setBalance(v as number)}
              onChangeCommitted={(_e, v) => props.onBalanceChange(v as number)}
              sx={sliderSx}
              valueLabelDisplay="auto"
              valueLabelFormat={(v) => `${v}%`}
              min={-100} max={100} step={1}
            />
          </div>
        </div>
      </div>

      {/* ── Stereo / channel ───────────────────────────────────────────── */}
      <div className="mb-4 text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Channel mode</div>
          <select
            value={channelConfig}
            onChange={(e) => {
              const v = Number(e.target.value);
              setChannelConfig(v);
              props.onChannelConfigChange(v);
            }}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {CHANNEL_MODES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Stereo width</div>
          <div className="flex items-center gap-2">
            <span className="text-[13px] text-[#aaa] w-10 text-right">{stereoWidth}%</span>
            <div style={{ width: 120 }}>
              <Slider
                value={stereoWidth}
                onChange={(_e, v) => setStereoWidth(v as number)}
                onChangeCommitted={(_e, v) => props.onStereoWidthChange(v as number)}
                sx={sliderSx}
                min={0} max={250} step={5}
              />
            </div>
          </div>
        </div>
      </div>

      {/* ── DSP extras ─────────────────────────────────────────────────── */}
      <div className="mb-4 text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Dithering</div>
          <Switch
            checked={props.ditheringEnabled}
            onChange={() => props.onDitheringChange(!props.ditheringEnabled)}
          />
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Auto FR correction</div>
          <select
            value={props.afrEnabled}
            onChange={(e) => props.onAfrChange(Number(e.target.value))}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {AFR_MODES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Bass enhancement (PBE)</div>
          <select
            value={props.pbe}
            onChange={(e) => props.onPbeChange(Number(e.target.value), pbePrecut)}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {PBE_MODES.map((label, i) => (
              <option key={i} value={i}>{label}</option>
            ))}
          </select>
        </div>
        {props.pbe > 0 && (
          <div className="flex flex-row items-center justify-between h-[50px]">
            <div>PBE pre-cut</div>
            <div className="flex items-center gap-2">
              <span className="text-[13px] text-[#aaa] w-14 text-right">
                {(pbePrecut / 10).toFixed(1)} dB
              </span>
              <div style={{ width: 120 }}>
                <Slider
                  value={pbePrecut}
                  onChange={(_e, v) => setPbePrecut(v as number)}
                  onChangeCommitted={(_e, v) => {
                    setPbePrecut(v as number);
                    props.onPbeChange(props.pbe, v as number);
                  }}
                  sx={sliderSx}
                  min={0} max={240} step={10}
                />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* ── Equalizer ──────────────────────────────────────────────────── */}
      <div className="mb-4 text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <EqualizerWithData />
      </div>

      {/* ── Section components ─────────────────────────────────────────── */}
      <CrossfeedWithData />
      <SurroundWithData />
      <CrossfadeWithData />
      <ReplayGainWithData />
    </>
  );
};

export default Sound;

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

// Surround enabled is a delay in ms; 0 = off.
const SURROUND_DELAYS = [0, 5, 8, 10, 15, 30] as const;

export type SurroundProps = {
  enabled: number;
  balance: number;
  fx1: number;
  fx2: number;
  mix: number;
  method2: boolean;
  onChange: (s: {
    enabled: number;
    balance: number;
    fx1: number;
    fx2: number;
    mix: number;
    method2: boolean;
  }) => void;
};

const Surround: FC<SurroundProps> = (props) => {
  const [enabled, setEnabled] = useState(props.enabled);
  const [balance, setBalance] = useState(props.balance);
  const [fx1, setFx1] = useState(props.fx1);
  const [fx2, setFx2] = useState(props.fx2);
  const [mix, setMix] = useState(props.mix);
  const [method2, setMethod2] = useState(props.method2);

  useEffect(() => {
    setEnabled(props.enabled);
    setBalance(props.balance);
    setFx1(props.fx1);
    setFx2(props.fx2);
    setMix(props.mix);
    setMethod2(props.method2);
  }, [props.enabled, props.balance, props.fx1, props.fx2, props.mix, props.method2]);

  const commit = (overrides?: Partial<SurroundProps>) => {
    props.onChange({ enabled, balance, fx1, fx2, mix, method2, ...overrides });
  };

  const isOn = enabled !== 0;

  return (
    <div className="mt-4">
      <div className="text-base font-semibold mb-2">Surround (Haas)</div>
      <div className="text-[15px] border border-[#8a8a8a65] rounded-[10px] px-5 py-[5px]">
        <div className="flex flex-row items-center justify-between h-[50px]">
          <div>Delay</div>
          <select
            value={enabled}
            onChange={(e) => {
              const v = Number(e.target.value);
              setEnabled(v);
              commit({ enabled: v });
            }}
            className="bg-transparent text-right text-[14px] outline-none cursor-pointer"
          >
            {SURROUND_DELAYS.map((d) => (
              <option key={d} value={d}>{d === 0 ? "Off" : `${d} ms`}</option>
            ))}
          </select>
        </div>

        {isOn && (
          <>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Balance</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-10 text-right">{balance}%</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={balance}
                    onChange={(_e, v) => setBalance(v as number)}
                    onChangeCommitted={(_e, v) => { setBalance(v as number); commit({ balance: v as number }); }}
                    sx={sliderSx}
                    min={0} max={99} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>LP cutoff</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-16 text-right">
                  {fx1 >= 1000 ? `${fx1 / 1000}kHz` : `${fx1}Hz`}
                </span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fx1}
                    onChange={(_e, v) => setFx1(v as number)}
                    onChangeCommitted={(_e, v) => { setFx1(v as number); commit({ fx1: v as number }); }}
                    sx={sliderSx}
                    min={600} max={8000} step={100}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>HP cutoff</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-16 text-right">{fx2} Hz</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={fx2}
                    onChange={(_e, v) => setFx2(v as number)}
                    onChangeCommitted={(_e, v) => { setFx2(v as number); commit({ fx2: v as number }); }}
                    sx={sliderSx}
                    min={40} max={400} step={10}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Mix</div>
              <div className="flex items-center gap-2">
                <span className="text-[13px] text-[#aaa] w-10 text-right">{mix}%</span>
                <div style={{ width: 120 }}>
                  <Slider
                    value={mix}
                    onChange={(_e, v) => setMix(v as number)}
                    onChangeCommitted={(_e, v) => { setMix(v as number); commit({ mix: v as number }); }}
                    sx={sliderSx}
                    min={0} max={100} step={1}
                  />
                </div>
              </div>
            </div>
            <div className="flex flex-row items-center justify-between h-[50px]">
              <div>Method 2</div>
              <Switch
                checked={method2}
                onChange={() => {
                  const v = !method2;
                  setMethod2(v);
                  commit({ method2: v });
                }}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default Surround;

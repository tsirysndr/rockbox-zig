import { FC, useEffect, useId, useRef, useState } from "react";
import Switch from "../../../Switch";
import { Slider } from "@mui/material";

const iOSBoxShadow =
  "0 3px 1px rgba(0,0,0,0.1),0 4px 8px rgba(0,0,0,0.13),0 0 0 1px rgba(0,0,0,0.02)";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sliderSx = (theme: any) => ({
  color: "#6F00FF",
  "& .MuiSlider-thumb": {
    height: 14,
    width: 14,
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
  "& .MuiSlider-track": { border: "none", width: 3 },
  "& .MuiSlider-rail": {
    opacity: 0.5,
    boxShadow: "inset 0px 0px 4px -2px #000",
    backgroundColor: "#d0d0d0",
    width: 3,
  },
  ...theme.applyStyles("dark", { color: "#6F00FF" }),
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const sliderSxHoriz = (theme: any) => ({
  color: "#6F00FF",
  "& .MuiSlider-thumb": {
    width: 14,
    height: 14,
    backgroundColor: "#fff",
    "&::before": { boxShadow: "0 4px 8px rgba(0,0,0,0.18)" },
    "&:hover, &.Mui-focusVisible, &.Mui-active": { boxShadow: "none" },
  },
  "& .MuiSlider-track": { border: "none", height: 3 },
  "& .MuiSlider-rail": {
    opacity: 0.5,
    boxShadow: "inset 0px 0px 4px -2px #000",
    backgroundColor: "#d0d0d0",
    height: 3,
  },
  ...theme.applyStyles("dark", { color: "#6F00FF" }),
});

function formatFreq(hz: number): string {
  if (hz >= 1000) return `${hz / 1000}kHz`;
  return `${hz}Hz`;
}

function formatGain(tenths: number): string {
  const db = (tenths / 10).toFixed(1);
  return tenths >= 0 ? `+${db}` : db;
}

// SVG coordinate space: 310 px tall canvas.
// TRACK_TOP / TRACK_BOTTOM are the y-pixel positions where MUI's thumb
// center sits at max (+24 dB) and min (-24 dB) gain inside the container.
// These are calibrated for thumb height=14 px and ~20 px of label padding.
const CONTAINER_H = 310;
const TRACK_TOP = 27;
const TRACK_BOTTOM = 283;
const TRACK_RANGE = TRACK_BOTTOM - TRACK_TOP;

function gainToY(gain: number): number {
  return TRACK_TOP + ((240 - gain) / 480) * TRACK_RANGE;
}

/** Catmull-Rom → cubic Bézier: smooth curve through band gain points. */
function catmullRom(pts: { x: number; y: number }[]): string {
  if (pts.length === 0) return "";
  if (pts.length === 1) return `M ${pts[0].x} ${pts[0].y}`;
  let d = `M ${pts[0].x.toFixed(1)} ${pts[0].y.toFixed(1)}`;
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)];
    const p1 = pts[i];
    const p2 = pts[i + 1];
    const p3 = pts[Math.min(pts.length - 1, i + 2)];
    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;
    d += ` C ${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p2.x.toFixed(1)},${p2.y.toFixed(1)}`;
  }
  return d;
}

const DB_GRID = [240, 120, 0, -120, -240] as const;

export type EqualizerProps = {
  eqEnabled: boolean;
  eqPrecut: number;
  onEnableEq: (enabled: boolean) => void;
  onEqPrecutChange: (precut: number) => void;
  eqBandSettings: { q: number; gain: number; cutoff: number }[];
  onEqBandSettingsChange: (
    bandSettings: { q: number; gain: number; cutoff: number }[]
  ) => void;
};

const Equalizer: FC<EqualizerProps> = (props) => {
  const [eqEnabled, setEqEnabled] = useState(props.eqEnabled);
  const [eqPrecut, setEqPrecut] = useState(props.eqPrecut);
  const [bands, setBands] = useState(props.eqBandSettings);

  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(600);

  // One ref per band column — used to measure exact slider-center x positions.
  const bandColRefs = useRef<(HTMLDivElement | null)[]>([]);
  const [bandXPositions, setBandXPositions] = useState<number[]>([]);

  const rawUid = useId();
  const uid = rawUid.replace(/:/g, "");

  useEffect(() => {
    setEqEnabled(props.eqEnabled);
    setEqPrecut(props.eqPrecut);
    setBands(props.eqBandSettings);
  }, [props.eqEnabled, props.eqPrecut, props.eqBandSettings]);

  // Track container width for SVG sizing.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(([entry]) =>
      setContainerWidth(entry.contentRect.width)
    );
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Measure the horizontal center of every band column after each layout change.
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const containerLeft = container.getBoundingClientRect().left;
    const positions = bandColRefs.current.map((el) => {
      if (!el) return 0;
      const r = el.getBoundingClientRect();
      return r.left - containerLeft + r.width / 2;
    });
    setBandXPositions(positions);
  }, [bands.length, containerWidth]);

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

  // --- SVG curve data ---
  const centerY = gainToY(0);

  // Use DOM-measured x; fall back to proportional estimate on first paint.
  const bandPoints = bands.map((band, i) => ({
    x:
      bandXPositions[i] !== undefined
        ? bandXPositions[i]
        : ((i + 0.5) / bands.length) * containerWidth,
    y: gainToY(band.gain),
  }));

  const curvePath = catmullRom(bandPoints);
  const first = bandPoints[0];
  const last = bandPoints[bandPoints.length - 1];

  // Area is bounded by the smooth curve on one side and the 0 dB line on the other.
  const areaPath =
    curvePath && bandPoints.length > 0
      ? `${curvePath} L ${last.x.toFixed(1)},${centerY.toFixed(1)} L ${first.x.toFixed(1)},${centerY.toFixed(1)} Z`
      : "";

  const gradBoostId = `eqGradBoost${uid}`;
  const gradCutId = `eqGradCut${uid}`;
  const clipUpperId = `eqClipUpper${uid}`;
  const clipLowerId = `eqClipLower${uid}`;

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

      {/* EQ Pre-cut */}
      <div className="flex flex-row items-center justify-between h-[50px]">
        <div>Pre-cut</div>
        <div className="flex items-center gap-2">
          <span className="text-[13px] text-[#aaa] w-14 text-right">
            {(eqPrecut / 10).toFixed(1)} dB
          </span>
          <div style={{ width: 120 }}>
            <Slider
              value={eqPrecut}
              onChange={(_e, v) => setEqPrecut(v as number)}
              onChangeCommitted={(_e, v) => {
                setEqPrecut(v as number);
                props.onEqPrecutChange(v as number);
              }}
              sx={sliderSxHoriz}
              min={0}
              max={240}
              step={5}
            />
          </div>
        </div>
      </div>

      {/* Vertical band sliders with SVG frequency-response overlay */}
      <div
        ref={containerRef}
        className="relative mx-auto mt-8 mb-4 w-full"
        style={{ height: CONTAINER_H }}
      >
        {/* Glassmorphism backing panel */}
        <div
          className="absolute inset-0 rounded-xl"
          style={{
            background: "rgba(111, 0, 255, 0.05)",
            border: "1px solid rgba(111, 0, 255, 0.15)",
            boxShadow: "0 0 32px 0 rgba(111,0,255,0.08) inset",
          }}
        />

        {/* Interactive band columns — rendered below the SVG overlay so
            the sliders remain fully clickable (SVG uses pointer-events:none). */}
        <div className="absolute inset-0 flex flex-row justify-between w-full h-full text-[11px]">
          {bands.map((band, index) => (
            <div
              key={index}
              ref={(el) => {
                bandColRefs.current[index] = el;
              }}
              className="flex flex-col items-center gap-1"
              style={{ height: "100%" }}
            >
              <div className="text-center text-[11px] text-[#ccc] font-mono leading-none mb-1">
                {formatGain(band.gain)}
              </div>
              <Slider
                value={band.gain}
                onChange={(_e, v) => handleGainChange(v as number, index)}
                onChangeCommitted={(_e, v) =>
                  handleGainCommit(v as number, index)
                }
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

        {/* SVG chart — visual only, pointer-events: none */}
        <svg
          className="absolute inset-0 pointer-events-none"
          width={containerWidth}
          height={CONTAINER_H}
        >
          <defs>
            {/* Boost: transparent at 0 dB → rich violet upward */}
            <linearGradient
              id={gradBoostId}
              x1="0"
              y1={centerY}
              x2="0"
              y2={TRACK_TOP}
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0%" stopColor="#6F00FF" stopOpacity={0} />
              <stop offset="35%" stopColor="#8b2fff" stopOpacity={0.5} />
              <stop offset="75%" stopColor="#a855f7" stopOpacity={0.72} />
              <stop offset="100%" stopColor="#c084fc" stopOpacity={0.85} />
            </linearGradient>

            {/* Cut: transparent at 0 dB → deep indigo downward */}
            <linearGradient
              id={gradCutId}
              x1="0"
              y1={centerY}
              x2="0"
              y2={TRACK_BOTTOM}
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0%" stopColor="#6F00FF" stopOpacity={0} />
              <stop offset="40%" stopColor="#7c3aed" stopOpacity={0.45} />
              <stop offset="100%" stopColor="#6d28d9" stopOpacity={0.65} />
            </linearGradient>

            {/* Clips for the two fill regions */}
            <clipPath id={clipUpperId}>
              <rect x={0} y={0} width={containerWidth} height={centerY} />
            </clipPath>
            <clipPath id={clipLowerId}>
              <rect
                x={0}
                y={centerY}
                width={containerWidth}
                height={CONTAINER_H - centerY}
              />
            </clipPath>
          </defs>

          {/* dB reference grid */}
          {DB_GRID.map((gain) => {
            const y = gainToY(gain);
            const isZero = gain === 0;
            const label =
              gain > 0
                ? `+${gain / 10}dB`
                : gain === 0
                  ? "0dB"
                  : `${gain / 10}dB`;
            return (
              <g key={gain}>
                <line
                  x1={0}
                  y1={y}
                  x2={containerWidth}
                  y2={y}
                  stroke="white"
                  strokeOpacity={isZero ? 0.22 : 0.08}
                  strokeWidth={isZero ? 1.5 : 1}
                  strokeDasharray={isZero ? undefined : "3 7"}
                />
                <text
                  x={6}
                  y={y - 4}
                  fontSize={9}
                  fill="white"
                  fillOpacity={isZero ? 0.45 : 0.3}
                  fontFamily="monospace"
                >
                  {label}
                </text>
              </g>
            );
          })}

          {/* Boost fill — curve-to-centerY area, clipped to upper half */}
          {areaPath && (
            <path
              d={areaPath}
              fill={`url(#${gradBoostId})`}
              clipPath={`url(#${clipUpperId})`}
            />
          )}

          {/* Cut fill — curve-to-centerY area, clipped to lower half */}
          {areaPath && (
            <path
              d={areaPath}
              fill={`url(#${gradCutId})`}
              clipPath={`url(#${clipLowerId})`}
            />
          )}

          {/* Outer diffuse glow */}
          {curvePath && (
            <path
              d={curvePath}
              fill="none"
              stroke="#6F00FF"
              strokeWidth={16}
              strokeOpacity={0.14}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          )}

          {/* Mid glow */}
          {curvePath && (
            <path
              d={curvePath}
              fill="none"
              stroke="#9b6dff"
              strokeWidth={6}
              strokeOpacity={0.38}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          )}

          {/* Sharp curve */}
          {curvePath && (
            <path
              d={curvePath}
              fill="none"
              stroke="#ddd6fe"
              strokeWidth={2}
              strokeOpacity={0.95}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          )}

          {/* Per-band dots — centered exactly on each slider track */}
          {bandPoints.map((pt, i) => (
            <g key={i}>
              <circle cx={pt.x} cy={pt.y} r={10} fill="#6F00FF" fillOpacity={0.18} />
              <circle cx={pt.x} cy={pt.y} r={5} fill="#a855f7" fillOpacity={0.35} />
              <circle
                cx={pt.x}
                cy={pt.y}
                r={3.5}
                fill="white"
                fillOpacity={0.97}
                stroke="#c084fc"
                strokeWidth={1.5}
              />
            </g>
          ))}
        </svg>
      </div>
    </>
  );
};

export default Equalizer;

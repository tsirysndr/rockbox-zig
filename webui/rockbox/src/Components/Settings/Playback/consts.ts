import {
  REPEAT_OFF,
  REPEAT_ALL,
  REPEAT_ONE,
  REPEAT_SHUFFLE,
  REPEAT_AB,
  CROSSFADE_OFF,
  CROSSFADE_ENABLE_AUTOSKIP,
  CROSSFADE_ENABLE_MANSKIP,
  CROSSFADE_ENABLE_SHUFFLE,
  CROSSFADE_ENABLE_SHUFFLE_OR_MANSKIP,
  CROSSFADE_ENABLE_ALWAYS,
  FADE_OUT_CROSSFADE_MODE,
  FADE_OUT_MIX_MODE,
  REPLAYGAIN_TRACK,
  REPLAYGAIN_ALBUM,
  REPLAYGAIN_SHUFFLE,
  REPLAYGAIN_OFF,
} from "../../../constants";

export const repeatOptions = [
  {
    label: "Off",
    id: "1",
  },
  {
    label: "All",
    id: "2",
  },
  {
    label: "One",
    id: "3",
  },
  {
    label: "Shuffle",
    id: "4",
  },
  {
    label: "A-B",
    id: "5",
  },
];

export const repeatValues: Record<number, { label: string; id: string }[]> = {
  [REPEAT_OFF]: [{ label: "Off", id: "1" }],
  [REPEAT_ALL]: [{ label: "All", id: "2" }],
  [REPEAT_ONE]: [{ label: "One", id: "3" }],
  [REPEAT_SHUFFLE]: [{ label: "Shuffle", id: "4" }],
  [REPEAT_AB]: [{ label: "A-B", id: "5" }],
};

export const crossfadeOptions = [
  {
    label: "Off",
    id: "1",
  },
  {
    label: "Automatic Track Change Only",
    id: "2",
  },
  {
    label: "Manual Track Change Only",
    id: "3",
  },
  {
    label: "Shuffle",
    id: "4",
  },
  {
    label: "Shuffle or Manual Track Skip",
    id: "5",
  },
  {
    label: "Always",
    id: "6",
  },
];

export const crossfadeValues: Record<number, { label: string; id: string }[]> =
  {
    [CROSSFADE_OFF]: [{ label: "Off", id: "1" }],
    [CROSSFADE_ENABLE_AUTOSKIP]: [
      { label: "Automatic Track Change Only", id: "2" },
    ],
    [CROSSFADE_ENABLE_MANSKIP]: [
      { label: "Manual Track Change Only", id: "3" },
    ],
    [CROSSFADE_ENABLE_SHUFFLE]: [{ label: "Shuffle", id: "4" }],
    [CROSSFADE_ENABLE_SHUFFLE_OR_MANSKIP]: [
      { label: "Shuffle or Manual Track Skip", id: "5" },
    ],
    [CROSSFADE_ENABLE_ALWAYS]: [{ label: "Always", id: "6" }],
  };

export const fadeOutModeOptions = [
  {
    label: "Crossfade",
    id: "1",
  },
  {
    label: "Mix",
    id: "2",
  },
];

export const fadeOutModeValues: Record<
  number,
  { label: string; id: string }[]
> = {
  [FADE_OUT_CROSSFADE_MODE]: [{ label: "Crossfade", id: "1" }],
  [FADE_OUT_MIX_MODE]: [{ label: "Mix", id: "2" }],
};

export const replaygainOptions = [
  {
    label: "Track Gain",
    id: "1",
  },
  {
    label: "Album Gain",
    id: "2",
  },
  {
    label: "Track Gain if Shuffling",
    id: "3",
  },
  {
    label: "Off",
    id: "4",
  },
];

export const replaygainValues: Record<number, { label: string; id: string }[]> =
  {
    [REPLAYGAIN_TRACK]: [{ label: "Track Gain", id: "1" }],
    [REPLAYGAIN_ALBUM]: [{ label: "Album Gain", id: "2" }],
    [REPLAYGAIN_SHUFFLE]: [{ label: "Track Gain if Shuffling", id: "3" }],
    [REPLAYGAIN_OFF]: [{ label: "Off", id: "4" }],
  };

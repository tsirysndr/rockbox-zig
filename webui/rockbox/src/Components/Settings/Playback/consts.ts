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
    value: REPEAT_OFF,
  },
  {
    label: "All",
    id: "2",
    value: REPEAT_ALL,
  },
  {
    label: "One",
    id: "3",
    value: REPEAT_ONE,
  },
  {
    label: "Shuffle",
    id: "4",
    value: REPEAT_SHUFFLE,
  },
  {
    label: "A-B",
    id: "5",
    value: REPEAT_AB,
  },
];

export const repeatValues: Record<
  number,
  { label: string; id: string; value: number }[]
> = {
  [REPEAT_OFF]: [{ label: "Off", id: "1", value: REPEAT_OFF }],
  [REPEAT_ALL]: [{ label: "All", id: "2", value: REPEAT_ALL }],
  [REPEAT_ONE]: [{ label: "One", id: "3", value: REPEAT_ONE }],
  [REPEAT_SHUFFLE]: [{ label: "Shuffle", id: "4", value: REPEAT_SHUFFLE }],
  [REPEAT_AB]: [{ label: "A-B", id: "5", value: REPEAT_AB }],
};

export const crossfadeOptions = [
  {
    label: "Off",
    id: "1",
    value: CROSSFADE_OFF,
  },
  {
    label: "Automatic Track Change Only",
    id: "2",
    value: CROSSFADE_ENABLE_AUTOSKIP,
  },
  {
    label: "Manual Track Change Only",
    id: "3",
    value: CROSSFADE_ENABLE_MANSKIP,
  },
  {
    label: "Shuffle",
    id: "4",
    value: CROSSFADE_ENABLE_SHUFFLE,
  },
  {
    label: "Shuffle or Manual Track Skip",
    id: "5",
    value: CROSSFADE_ENABLE_SHUFFLE_OR_MANSKIP,
  },
  {
    label: "Always",
    id: "6",
    value: CROSSFADE_ENABLE_ALWAYS,
  },
];

export const crossfadeValues: Record<
  number,
  { label: string; id: string; value: number }[]
> = {
  [CROSSFADE_OFF]: [{ label: "Off", id: "1", value: CROSSFADE_OFF }],
  [CROSSFADE_ENABLE_AUTOSKIP]: [
    {
      label: "Automatic Track Change Only",
      id: "2",
      value: CROSSFADE_ENABLE_AUTOSKIP,
    },
  ],
  [CROSSFADE_ENABLE_MANSKIP]: [
    {
      label: "Manual Track Change Only",
      id: "3",
      value: CROSSFADE_ENABLE_MANSKIP,
    },
  ],
  [CROSSFADE_ENABLE_SHUFFLE]: [
    { label: "Shuffle", id: "4", value: CROSSFADE_ENABLE_SHUFFLE },
  ],
  [CROSSFADE_ENABLE_SHUFFLE_OR_MANSKIP]: [
    {
      label: "Shuffle or Manual Track Skip",
      id: "5",
      value: CROSSFADE_ENABLE_SHUFFLE_OR_MANSKIP,
    },
  ],
  [CROSSFADE_ENABLE_ALWAYS]: [
    { label: "Always", id: "6", value: CROSSFADE_ENABLE_ALWAYS },
  ],
};

export const fadeOutModeOptions = [
  {
    label: "Crossfade",
    id: "1",
    value: FADE_OUT_CROSSFADE_MODE,
  },
  {
    label: "Mix",
    id: "2",
    value: FADE_OUT_MIX_MODE,
  },
];

export const fadeOutModeValues: Record<
  number,
  { label: string; id: string; value: number }[]
> = {
  [FADE_OUT_CROSSFADE_MODE]: [
    { label: "Crossfade", id: "1", value: FADE_OUT_CROSSFADE_MODE },
  ],
  [FADE_OUT_MIX_MODE]: [{ label: "Mix", id: "2", value: FADE_OUT_MIX_MODE }],
};

export const replaygainOptions = [
  {
    label: "Track Gain",
    id: "1",
    value: REPLAYGAIN_TRACK,
  },
  {
    label: "Album Gain",
    id: "2",
    value: REPLAYGAIN_ALBUM,
  },
  {
    label: "Track Gain if Shuffling",
    id: "3",
    value: REPLAYGAIN_SHUFFLE,
  },
  {
    label: "Off",
    id: "4",
    value: REPLAYGAIN_OFF,
  },
];

export const replaygainValues: Record<
  number,
  { label: string; id: string; value: number }[]
> = {
  [REPLAYGAIN_TRACK]: [
    { label: "Track Gain", id: "1", value: REPLAYGAIN_TRACK },
  ],
  [REPLAYGAIN_ALBUM]: [
    { label: "Album Gain", id: "2", value: REPLAYGAIN_ALBUM },
  ],
  [REPLAYGAIN_SHUFFLE]: [
    { label: "Track Gain if Shuffling", id: "3", value: REPLAYGAIN_SHUFFLE },
  ],
  [REPLAYGAIN_OFF]: [{ label: "Off", id: "4", value: REPLAYGAIN_OFF }],
};

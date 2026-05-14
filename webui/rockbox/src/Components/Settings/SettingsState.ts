import { atom } from "recoil";

export const settingsState = atom<{
  enableBlur: boolean;
  eqBandSettings: {
    q: number;
    gain: number;
    cutoff: number;
  }[];
  eqEnabled: boolean;
  eqPrecut: number;
  volume: number;
  bass: number;
  bassCutoff: number;
  treble: number;
  trebleCutoff: number;
  playlistShuffle: boolean;
  repeatMode: number;
  replaygainSettings: {
    noclip: boolean;
    type: number;
    preamp: number;
  };
  playerName: string;
  partyMode: boolean;
  ditheringEnabled: boolean;
  channelConfig: number;
  balance: number;
  crossfade: number;
  fadeOnStop: boolean;
  crossfadeFadeInDelay: number;
  crossfadeFadeInDuration: number;
  crossfadeFadeOutDelay: number;
  crossfadeFadeOutDuration: number;
  crossfadeFadeOutMixmode: number;
  stereoWidth: number;
  stereoswMode: number;
  surroundEnabled: number;
  surroundBalance: number;
  surroundFx1: number;
  surroundFx2: number;
  surroundMix: number;
  surroundMethod2: boolean;
  crossfeedType: number;
  crossfeedDirectGain: number;
  crossfeedCrossGain: number;
  crossfeedHfAttenuation: number;
  crossfeedHfCutoff: number;
  afrEnabled: number;
  pbe: number;
  pbePrecut: number;
}>({
  key: "settings",
  default: {
    enableBlur: false,
    eqBandSettings: [
      { cutoff: 60,    q: 70, gain: 0 },
      { cutoff: 200,   q: 70, gain: 0 },
      { cutoff: 500,   q: 70, gain: 0 },
      { cutoff: 1000,  q: 70, gain: 0 },
      { cutoff: 2000,  q: 70, gain: 0 },
      { cutoff: 4000,  q: 70, gain: 0 },
      { cutoff: 7000,  q: 70, gain: 0 },
      { cutoff: 10000, q: 70, gain: 0 },
      { cutoff: 14000, q: 70, gain: 0 },
      { cutoff: 20000, q: 70, gain: 0 },
    ],
    eqEnabled: false,
    eqPrecut: 0,
    volume: 0,
    bass: 0,
    bassCutoff: 0,
    treble: 0,
    trebleCutoff: 0,
    playlistShuffle: false,
    repeatMode: 0,
    replaygainSettings: {
      noclip: false,
      type: 0,
      preamp: 0,
    },
    playerName: "",
    partyMode: false,
    ditheringEnabled: false,
    channelConfig: 0,
    balance: 0,
    stereoswMode: 0,
    stereoWidth: 0,
    surroundEnabled: 0,
    surroundBalance: 50,
    surroundFx1: 1200,
    surroundFx2: 100,
    surroundMix: 100,
    surroundMethod2: false,
    crossfade: 0,
    fadeOnStop: false,
    crossfadeFadeInDelay: 0,
    crossfadeFadeInDuration: 8,
    crossfadeFadeOutDelay: 0,
    crossfadeFadeOutDuration: 8,
    crossfadeFadeOutMixmode: 0,
    crossfeedType: 0,
    crossfeedDirectGain: -115,
    crossfeedCrossGain: -320,
    crossfeedHfAttenuation: -160,
    crossfeedHfCutoff: 700,
    afrEnabled: 0,
    pbe: 0,
    pbePrecut: 0,
  },
});

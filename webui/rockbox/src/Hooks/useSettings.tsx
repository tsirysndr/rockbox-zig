import { useRecoilState } from "recoil";
import { settingsState } from "../Components/Settings/SettingsState";
import { useGetGlobalSettingsQuery } from "./GraphQL";
import { useEffect } from "react";

// Matches the WASM EQ preset exactly: 10-band layout with Q=7.0 (stored as 70).
// The Rockbox firmware ships with different defaults (32/64/125 Hz, Q=0.7/1.0)
// which produce much wider, softer filters than the WASM EQ.  We always enforce
// the WASM cutoff + Q here and only read gain from global_settings.
const EQ_BAND_PRESETS = [
  { cutoff: 60,    q: 70 },
  { cutoff: 200,   q: 70 },
  { cutoff: 500,   q: 70 },
  { cutoff: 1000,  q: 70 },
  { cutoff: 2000,  q: 70 },
  { cutoff: 4000,  q: 70 },
  { cutoff: 7000,  q: 70 },
  { cutoff: 10000, q: 70 },
  { cutoff: 14000, q: 70 },
  { cutoff: 20000, q: 70 },
] as const;

export const useSettings = () => {
  const [, setSettings] = useRecoilState(settingsState);
  const { data, isLoading } = useGetGlobalSettingsQuery();

  useEffect(() => {
    if (!data || isLoading) {
      return;
    }
    setSettings((state) => ({
      ...state,
      eqEnabled: data.globalSettings.eqEnabled,
      eqBandSettings: data.globalSettings.eqBandSettings.map((band, i) => ({
        cutoff: EQ_BAND_PRESETS[i].cutoff,
        q: EQ_BAND_PRESETS[i].q,
        gain: band.gain,
      })),
      volume: data.globalSettings.volume,
      bass: data.globalSettings.bass,
      bassCutoff: data.globalSettings.bassCutoff,
      treble: data.globalSettings.treble,
      trebleCutoff: data.globalSettings.trebleCutoff,
      playlistShuffle: data.globalSettings.playlistShuffle,
      repeatMode: data.globalSettings.repeatMode,
      replaygainSettings: data.globalSettings.replaygainSettings,
      playerName: data.globalSettings.playerName,
      partyMode: data.globalSettings.partyMode,
      ditheringEnabled: data.globalSettings.ditheringEnabled,
      channelConfig: data.globalSettings.channelConfig,
      balance: data.globalSettings.balance,
      fadeOnStop: data.globalSettings.fadeOnStop,
      crossfade: data.globalSettings.crossfade,
      crossfadeFadeInDelay: data.globalSettings.crossfadeFadeInDelay,
      crossfadeFadeInDuration: data.globalSettings.crossfadeFadeInDuration,
      crossfadeFadeOutDelay: data.globalSettings.crossfadeFadeOutDelay,
      crossfadeFadeOutDuration: data.globalSettings.crossfadeFadeOutDuration,
      crossfadeFadeOutMixmode: data.globalSettings.crossfadeFadeOutMixmode,
      stereoWidth: data.globalSettings.stereoWidth,
      stereoswMode: data.globalSettings.stereoswMode,
      surroundEnabled: data.globalSettings.surroundEnabled,
      surroundBalance: data.globalSettings.surroundBalance,
      surroundFx1: data.globalSettings.surroundFx1,
      surroundFx2: data.globalSettings.surroundFx2,
      surroundMix: data.globalSettings.surroundMix,
      surroundMethod2: data.globalSettings.surroundMethod2,
      crossfeedType: data.globalSettings.crossfeed,
      crossfeedDirectGain: data.globalSettings.crossfeedDirectGain,
      crossfeedCrossGain: data.globalSettings.crossfeedCrossGain,
      crossfeedHfAttenuation: data.globalSettings.crossfeedHfAttenuation,
      crossfeedHfCutoff: data.globalSettings.crossfeedHfCutoff,
      eqPrecut: data.globalSettings.eqPrecut,
      afrEnabled: data.globalSettings.afrEnabled,
      pbe: data.globalSettings.pbe,
      pbePrecut: data.globalSettings.pbePrecut,
    }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, isLoading]);
};

import { FC, useEffect } from "react";
import Settings from "./Settings";
import { useGetGlobalSettingsQuery } from "../../Hooks/GraphQL";
import { useRecoilState } from "recoil";
import { settingsState } from "./SettingsState";

const SettingsWithData: FC = () => {
  const [, setSettings] = useRecoilState(settingsState);
  const { data, loading } = useGetGlobalSettingsQuery();

  useEffect(() => {
    if (!data || loading) {
      return;
    }
    setSettings((state) => ({
      ...state,
      eqEnabled: data.globalSettings.eqEnabled,
      eqBandSettings: data.globalSettings.eqBandSettings,
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
    }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, loading]);

  return <Settings />;
};

export default SettingsWithData;

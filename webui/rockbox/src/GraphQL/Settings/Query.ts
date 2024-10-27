import { gql } from "@apollo/client";

export const GET_GLOBAL_SETTINGS = gql`
  query GetGlobalSettings {
    globalSettings {
      volume
      playlistShuffle
      repeatMode
      bass
      bassCutoff
      treble
      trebleCutoff
      crossfade
      fadeOnStop
      crossfadeFadeInDelay
      crossfadeFadeInDuration
      crossfadeFadeOutDelay
      crossfadeFadeOutDuration
      crossfadeFadeOutMixmode
      balance
      stereoWidth
      stereoswMode
      surroundEnabled
      surroundBalance
      surroundFx1
      surroundFx2
      partyMode
      ditheringEnabled
      channelConfig
      playerName
      eqEnabled
      eqBandSettings {
        q
        cutoff
        gain
      }
      replaygainSettings {
        noclip
        type
        preamp
      }
    }
  }
`;

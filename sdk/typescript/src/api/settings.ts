import type { HttpTransport } from '../transport.js';
import type { UserSettings, PartialUserSettings } from '../types.js';

export class SettingsApi {
  constructor(private readonly http: HttpTransport) {}

  async get(): Promise<UserSettings> {
    const data = await this.http.execute<{ globalSettings: UserSettings }>(/* GraphQL */ `
      query GlobalSettings {
        globalSettings {
          musicDir volume balance bass treble channelConfig stereoWidth
          eqEnabled eqPrecut
          eqBandSettings { cutoff q gain }
          replaygainSettings { noclip type preamp }
          compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
          crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
          crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
          crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
          crossfeedHfAttenuation crossfeedHfCutoff
          repeatMode singleMode partyMode shuffle playerName
        }
      }
    `);
    return data.globalSettings;
  }

  async save(settings: PartialUserSettings): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation SaveSettings($settings: NewGlobalSettings!) { saveSettings(settings: $settings) }
    `, { settings });
  }
}

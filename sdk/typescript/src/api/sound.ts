import type { HttpTransport } from '../transport.js';
import type { VolumeInfo } from '../types.js';

export class SoundApi {
  constructor(private readonly http: HttpTransport) {}

  /** Get current volume with min/max range */
  async getVolume(): Promise<VolumeInfo> {
    const data = await this.http.execute<{ volume: VolumeInfo }>(/* GraphQL */ `
      query Volume { volume { volume min max } }
    `);
    return data.volume;
  }

  /** Adjust volume by a relative number of steps (positive = louder, negative = quieter) */
  async adjustVolume(steps: number): Promise<number> {
    const data = await this.http.execute<{ adjustVolume: number }>(/* GraphQL */ `
      mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }
    `, { steps });
    return data.adjustVolume;
  }

  /** Increase volume by one step */
  async volumeUp(): Promise<number> {
    return this.adjustVolume(1);
  }

  /** Decrease volume by one step */
  async volumeDown(): Promise<number> {
    return this.adjustVolume(-1);
  }
}

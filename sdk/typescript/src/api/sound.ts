import type { HttpTransport } from '../transport.js';

export class SoundApi {
  constructor(private readonly http: HttpTransport) {}

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

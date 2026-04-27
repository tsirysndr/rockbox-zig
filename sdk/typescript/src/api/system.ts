import type { HttpTransport } from '../transport.js';
import type { SystemStatus } from '../types.js';

export class SystemApi {
  constructor(private readonly http: HttpTransport) {}

  async version(): Promise<string> {
    const data = await this.http.execute<{ rockboxVersion: string }>(/* GraphQL */ `
      query Version { rockboxVersion }
    `);
    return data.rockboxVersion;
  }

  async status(): Promise<SystemStatus> {
    const data = await this.http.execute<{ globalStatus: SystemStatus }>(/* GraphQL */ `
      query GlobalStatus {
        globalStatus {
          resumeIndex resumeCrc32 resumeElapsed resumeOffset
          runtime topruntime dircacheSize
          lastScreen viewerIconCount lastVolumeChange
        }
      }
    `);
    return data.globalStatus;
  }
}

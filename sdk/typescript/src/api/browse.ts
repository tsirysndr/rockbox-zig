import type { HttpTransport } from '../transport.js';
import type { Entry } from '../types.js';
import { isDirectory } from '../types.js';

export class BrowseApi {
  constructor(private readonly http: HttpTransport) {}

  async entries(path?: string): Promise<Entry[]> {
    const data = await this.http.execute<{ treeGetEntries: Entry[] }>(/* GraphQL */ `
      query Browse($path: String) {
        treeGetEntries(path: $path) { name attr timeWrite customaction }
      }
    `, { path });
    return data.treeGetEntries;
  }

  async directories(path?: string): Promise<Entry[]> {
    const all = await this.entries(path);
    return all.filter(isDirectory);
  }

  async files(path?: string): Promise<Entry[]> {
    const all = await this.entries(path);
    return all.filter((e) => !isDirectory(e));
  }
}

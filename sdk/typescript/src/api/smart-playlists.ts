import type { HttpTransport } from '../transport.js';
import type { SmartPlaylist, TrackStats } from '../types.js';

export interface CreateSmartPlaylistInput {
  name: string;
  rules: string;
  description?: string;
  image?: string;
  folderId?: string;
}

export interface UpdateSmartPlaylistInput {
  name: string;
  rules: string;
  description?: string;
  image?: string;
  folderId?: string;
}

export class SmartPlaylistsApi {
  constructor(private readonly http: HttpTransport) {}

  async list(): Promise<SmartPlaylist[]> {
    const data = await this.http.execute<{ smartPlaylists: SmartPlaylist[] }>(/* GraphQL */ `
      query SmartPlaylists {
        smartPlaylists {
          id name description image folderId isSystem rules createdAt updatedAt
        }
      }
    `);
    return data.smartPlaylists;
  }

  async get(id: string): Promise<SmartPlaylist | null> {
    const data = await this.http.execute<{ smartPlaylist: SmartPlaylist | null }>(/* GraphQL */ `
      query SmartPlaylist($id: String!) {
        smartPlaylist(id: $id) {
          id name description image folderId isSystem rules createdAt updatedAt
        }
      }
    `, { id });
    return data.smartPlaylist;
  }

  async trackIds(id: string): Promise<string[]> {
    const data = await this.http.execute<{ smartPlaylistTrackIds: string[] }>(/* GraphQL */ `
      query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }
    `, { id });
    return data.smartPlaylistTrackIds;
  }

  async create(input: CreateSmartPlaylistInput): Promise<SmartPlaylist> {
    const data = await this.http.execute<{ createSmartPlaylist: SmartPlaylist }>(/* GraphQL */ `
      mutation CreateSmartPlaylist(
        $name: String!, $rules: String!, $description: String,
        $image: String, $folderId: String
      ) {
        createSmartPlaylist(
          name: $name, rules: $rules, description: $description,
          image: $image, folderId: $folderId
        ) {
          id name description image folderId isSystem rules createdAt updatedAt
        }
      }
    `, input);
    return data.createSmartPlaylist;
  }

  async update(id: string, input: UpdateSmartPlaylistInput): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation UpdateSmartPlaylist(
        $id: String!, $name: String!, $rules: String!,
        $description: String, $image: String, $folderId: String
      ) {
        updateSmartPlaylist(
          id: $id, name: $name, rules: $rules, description: $description,
          image: $image, folderId: $folderId
        )
      }
    `, { id, ...input });
  }

  async delete(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }
    `, { id });
  }

  async play(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }
    `, { id });
  }

  // ---------------------------------------------------------------------------
  // Listening stats (feeds smart playlist rules)
  // ---------------------------------------------------------------------------

  async trackStats(trackId: string): Promise<TrackStats | null> {
    const data = await this.http.execute<{ trackStats: TrackStats | null }>(/* GraphQL */ `
      query TrackStats($trackId: String!) {
        trackStats(trackId: $trackId) {
          trackId playCount skipCount lastPlayed lastSkipped updatedAt
        }
      }
    `, { trackId });
    return data.trackStats;
  }

  async recordPlayed(trackId: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation RecordTrackPlayed($trackId: String!) { recordTrackPlayed(trackId: $trackId) }
    `, { trackId });
  }

  async recordSkipped(trackId: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation RecordTrackSkipped($trackId: String!) { recordTrackSkipped(trackId: $trackId) }
    `, { trackId });
  }
}

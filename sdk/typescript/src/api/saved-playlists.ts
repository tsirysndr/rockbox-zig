import type { HttpTransport } from '../transport.js';
import type { SavedPlaylist, SavedPlaylistFolder } from '../types.js';

export interface CreatePlaylistInput {
  name: string;
  description?: string;
  image?: string;
  folderId?: string;
  trackIds?: string[];
}

export interface UpdatePlaylistInput {
  name: string;
  description?: string;
  image?: string;
  folderId?: string;
}

export class SavedPlaylistsApi {
  constructor(private readonly http: HttpTransport) {}

  async list(folderId?: string): Promise<SavedPlaylist[]> {
    const data = await this.http.execute<{ savedPlaylists: SavedPlaylist[] }>(/* GraphQL */ `
      query SavedPlaylists($folderId: String) {
        savedPlaylists(folderId: $folderId) {
          id name description image folderId trackCount createdAt updatedAt
        }
      }
    `, { folderId });
    return data.savedPlaylists;
  }

  async get(id: string): Promise<SavedPlaylist | null> {
    const data = await this.http.execute<{ savedPlaylist: SavedPlaylist | null }>(/* GraphQL */ `
      query SavedPlaylist($id: String!) {
        savedPlaylist(id: $id) {
          id name description image folderId trackCount createdAt updatedAt
        }
      }
    `, { id });
    return data.savedPlaylist;
  }

  async trackIds(playlistId: string): Promise<string[]> {
    const data = await this.http.execute<{ savedPlaylistTrackIds: string[] }>(/* GraphQL */ `
      query SavedPlaylistTrackIds($playlistId: String!) {
        savedPlaylistTrackIds(playlistId: $playlistId)
      }
    `, { playlistId });
    return data.savedPlaylistTrackIds;
  }

  async create(input: CreatePlaylistInput): Promise<SavedPlaylist> {
    const data = await this.http.execute<{ createSavedPlaylist: SavedPlaylist }>(/* GraphQL */ `
      mutation CreateSavedPlaylist(
        $name: String!, $description: String, $image: String,
        $folderId: String, $trackIds: [String!]
      ) {
        createSavedPlaylist(
          name: $name, description: $description, image: $image,
          folderId: $folderId, trackIds: $trackIds
        ) {
          id name description image folderId trackCount createdAt updatedAt
        }
      }
    `, input);
    return data.createSavedPlaylist;
  }

  async update(id: string, input: UpdatePlaylistInput): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation UpdateSavedPlaylist(
        $id: String!, $name: String!, $description: String, $image: String, $folderId: String
      ) {
        updateSavedPlaylist(
          id: $id, name: $name, description: $description, image: $image, folderId: $folderId
        )
      }
    `, { id, ...input });
  }

  async delete(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }
    `, { id });
  }

  async addTracks(playlistId: string, trackIds: string[]): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) {
        addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds)
      }
    `, { playlistId, trackIds });
  }

  async removeTrack(playlistId: string, trackId: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) {
        removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId)
      }
    `, { playlistId, trackId });
  }

  async play(playlistId: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlaySavedPlaylist($playlistId: String!) { playSavedPlaylist(playlistId: $playlistId) }
    `, { playlistId });
  }

  // ---------------------------------------------------------------------------
  // Folders
  // ---------------------------------------------------------------------------

  async folders(): Promise<SavedPlaylistFolder[]> {
    const data = await this.http.execute<{ playlistFolders: SavedPlaylistFolder[] }>(/* GraphQL */ `
      query PlaylistFolders {
        playlistFolders { id name createdAt updatedAt }
      }
    `);
    return data.playlistFolders;
  }

  async createFolder(name: string): Promise<SavedPlaylistFolder> {
    const data = await this.http.execute<{ createPlaylistFolder: SavedPlaylistFolder }>(/* GraphQL */ `
      mutation CreatePlaylistFolder($name: String!) {
        createPlaylistFolder(name: $name) { id name createdAt updatedAt }
      }
    `, { name });
    return data.createPlaylistFolder;
  }

  async deleteFolder(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }
    `, { id });
  }
}

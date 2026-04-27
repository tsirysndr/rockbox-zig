import type { HttpTransport } from '../transport.js';
import type { Playlist } from '../types.js';
import { InsertPosition } from '../types.js';

const TRACK_FIELDS = /* GraphQL */ `
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
`;

export class PlaylistApi {
  constructor(private readonly http: HttpTransport) {}

  async current(): Promise<Playlist> {
    const data = await this.http.execute<{ playlistGetCurrent: Playlist }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query CurrentPlaylist {
        playlistGetCurrent {
          amount index maxPlaylistSize firstIndex
          lastInsertPos seed lastShuffledStart
          tracks { ...TrackFields }
        }
      }
    `);
    return data.playlistGetCurrent;
  }

  async amount(): Promise<number> {
    const data = await this.http.execute<{ playlistAmount: number }>(/* GraphQL */ `
      query PlaylistAmount { playlistAmount }
    `);
    return data.playlistAmount;
  }

  // ---------------------------------------------------------------------------
  // Queue management
  // ---------------------------------------------------------------------------

  /**
   * Insert tracks into the current playlist.
   * @param paths  File paths or track IDs to insert
   * @param position  Where to insert (default: Next after current)
   * @param playlistId  Target playlist ID; omit for the active queue
   */
  async insertTracks(
    paths: string[],
    position: InsertPosition = InsertPosition.Next,
    playlistId?: string,
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation InsertTracks($playlistId: String, $position: Int!, $tracks: [String!]!) {
        insertTracks(playlistId: $playlistId, position: $position, tracks: $tracks)
      }
    `, { playlistId, position, tracks: paths });
  }

  /** Insert a directory (optionally recursive) into the queue */
  async insertDirectory(
    directory: string,
    position: InsertPosition = InsertPosition.Last,
    playlistId?: string,
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation InsertDirectory($playlistId: String, $position: Int!, $directory: String!) {
        insertDirectory(playlistId: $playlistId, position: $position, directory: $directory)
      }
    `, { playlistId, position, directory });
  }

  /** Append all tracks from an album to the queue */
  async insertAlbum(albumId: string, position: InsertPosition = InsertPosition.Last): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation InsertAlbum($albumId: String!, $position: Int!) {
        insertAlbum(albumId: $albumId, position: $position)
      }
    `, { albumId, position });
  }

  async removeTrack(index: number): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation RemoveTrack($index: Int!) { playlistRemoveTrack(index: $index) }
    `, { index });
  }

  async clear(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation ClearPlaylist { playlistRemoveAllTracks }`);
  }

  async shuffle(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation ShufflePlaylist { shufflePlaylist }`);
  }

  /**
   * Create and start a new temporary playlist from a list of paths.
   * This replaces the current queue.
   */
  async create(name: string, tracks: string[]): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation CreatePlaylist($name: String!, $tracks: [String!]!) {
        playlistCreate(name: $name, tracks: $tracks)
      }
    `, { name, tracks });
  }

  async start(options: { startIndex?: number; elapsed?: number; offset?: number } = {}): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlaylistStart($startIndex: Int, $elapsed: Int, $offset: Int) {
        playlistStart(startIndex: $startIndex, elapsed: $elapsed, offset: $offset)
      }
    `, options);
  }

  async resume(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation PlaylistResume { playlistResume }`);
  }
}

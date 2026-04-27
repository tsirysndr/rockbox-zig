import type { HttpTransport } from '../transport.js';
import type { Track } from '../types.js';
import { PlaybackStatus } from '../types.js';

const TRACK_FIELDS = /* GraphQL */ `
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
`;

export class PlaybackApi {
  constructor(private readonly http: HttpTransport) {}

  /** Raw numeric playback status from the firmware */
  async rawStatus(): Promise<number> {
    const data = await this.http.execute<{ status: number }>(/* GraphQL */ `
      query PlaybackStatus { status }
    `);
    return data.status;
  }

  /** Typed playback status */
  async status(): Promise<PlaybackStatus> {
    return this.rawStatus() as Promise<PlaybackStatus>;
  }

  async currentTrack(): Promise<Track | null> {
    const data = await this.http.execute<{ currentTrack: Track | null }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query CurrentTrack { currentTrack { ...TrackFields } }
    `);
    return data.currentTrack;
  }

  async nextTrack(): Promise<Track | null> {
    const data = await this.http.execute<{ nextTrack: Track | null }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query NextTrack { nextTrack { ...TrackFields } }
    `);
    return data.nextTrack;
  }

  async filePosition(): Promise<number> {
    const data = await this.http.execute<{ getFilePosition: number }>(/* GraphQL */ `
      query FilePosition { getFilePosition }
    `);
    return data.getFilePosition;
  }

  // ---------------------------------------------------------------------------
  // Transport controls
  // ---------------------------------------------------------------------------

  async play(elapsed = 0, offset = 0): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation Play($elapsed: Long!, $offset: Long!) { play(elapsed: $elapsed, offset: $offset) }
    `, { elapsed, offset });
  }

  async pause(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation Pause { pause }`);
  }

  async resume(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation Resume { resume }`);
  }

  async next(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation Next { next }`);
  }

  async previous(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation Previous { previous }`);
  }

  /** Seek to an absolute position in milliseconds */
  async seek(positionMs: number): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation Seek($newTime: Int!) { fastForwardRewind(newTime: $newTime) }
    `, { newTime: positionMs });
  }

  async stop(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation Stop { hardStop }`);
  }

  /** Reload and flush the current track queue */
  async flushAndReload(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation FlushReload { flushAndReloadTracks }`);
  }

  // ---------------------------------------------------------------------------
  // Play helpers — single-call shortcuts (inspired by Navidrome & Kodi)
  // ---------------------------------------------------------------------------

  async playTrack(path: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayTrack($path: String!) { playTrack(path: $path) }
    `, { path });
  }

  async playAlbum(
    albumId: string,
    options: { shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayAlbum($albumId: String!, $shuffle: Boolean, $position: Int) {
        playAlbum(albumId: $albumId, shuffle: $shuffle, position: $position)
      }
    `, { albumId, ...options });
  }

  async playArtist(
    artistId: string,
    options: { shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayArtist($artistId: String!, $shuffle: Boolean, $position: Int) {
        playArtistTracks(artistId: $artistId, shuffle: $shuffle, position: $position)
      }
    `, { artistId, ...options });
  }

  async playPlaylist(
    playlistId: string,
    options: { shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayPlaylist($playlistId: String!, $shuffle: Boolean, $position: Int) {
        playPlaylist(playlistId: $playlistId, shuffle: $shuffle, position: $position)
      }
    `, { playlistId, ...options });
  }

  async playDirectory(
    path: string,
    options: { recurse?: boolean; shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayDirectory($path: String!, $recurse: Boolean, $shuffle: Boolean, $position: Int) {
        playDirectory(path: $path, recurse: $recurse, shuffle: $shuffle, position: $position)
      }
    `, { path, ...options });
  }

  async playLikedTracks(
    options: { shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayLikedTracks($shuffle: Boolean, $position: Int) {
        playLikedTracks(shuffle: $shuffle, position: $position)
      }
    `, options);
  }

  async playAllTracks(
    options: { shuffle?: boolean; position?: number } = {},
  ): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation PlayAllTracks($shuffle: Boolean, $position: Int) {
        playAllTracks(shuffle: $shuffle, position: $position)
      }
    `, options);
  }
}

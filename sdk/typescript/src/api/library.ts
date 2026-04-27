import type { HttpTransport } from '../transport.js';
import type { Track, Album, Artist, SearchResults } from '../types.js';

const TRACK_FIELDS = /* GraphQL */ `
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
`;

const ALBUM_FIELDS = /* GraphQL */ `
  fragment AlbumFields on Album {
    id title artist year yearString albumArt md5 artistId
  }
`;

const ARTIST_FIELDS = /* GraphQL */ `
  fragment ArtistFields on Artist {
    id name bio image
  }
`;

export class LibraryApi {
  constructor(private readonly http: HttpTransport) {}

  // ---------------------------------------------------------------------------
  // Albums
  // ---------------------------------------------------------------------------

  async albums(): Promise<Album[]> {
    const data = await this.http.execute<{ albums: Album[] }>(/* GraphQL */ `
      ${ALBUM_FIELDS}
      query Albums { albums { ...AlbumFields tracks { id title path length albumArt } } }
    `);
    return data.albums;
  }

  async album(id: string): Promise<Album | null> {
    const data = await this.http.execute<{ album: Album | null }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      ${ALBUM_FIELDS}
      query Album($id: String!) {
        album(id: $id) { ...AlbumFields tracks { ...TrackFields } }
      }
    `, { id });
    return data.album;
  }

  async likedAlbums(): Promise<Album[]> {
    const data = await this.http.execute<{ likedAlbums: Album[] }>(/* GraphQL */ `
      ${ALBUM_FIELDS}
      query LikedAlbums { likedAlbums { ...AlbumFields } }
    `);
    return data.likedAlbums;
  }

  async likeAlbum(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }
    `, { id });
  }

  async unlikeAlbum(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }
    `, { id });
  }

  // ---------------------------------------------------------------------------
  // Artists
  // ---------------------------------------------------------------------------

  async artists(): Promise<Artist[]> {
    const data = await this.http.execute<{ artists: Artist[] }>(/* GraphQL */ `
      ${ARTIST_FIELDS}
      query Artists { artists { ...ArtistFields albums { id title albumArt year } } }
    `);
    return data.artists;
  }

  async artist(id: string): Promise<Artist | null> {
    const data = await this.http.execute<{ artist: Artist | null }>(/* GraphQL */ `
      ${ARTIST_FIELDS}
      ${TRACK_FIELDS}
      query Artist($id: String!) {
        artist(id: $id) {
          ...ArtistFields
          albums { id title albumArt year yearString md5 artistId tracks { id title path length } }
          tracks { ...TrackFields }
        }
      }
    `, { id });
    return data.artist;
  }

  // ---------------------------------------------------------------------------
  // Tracks
  // ---------------------------------------------------------------------------

  async tracks(): Promise<Track[]> {
    const data = await this.http.execute<{ tracks: Track[] }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query Tracks { tracks { ...TrackFields } }
    `);
    return data.tracks;
  }

  async track(id: string): Promise<Track | null> {
    const data = await this.http.execute<{ track: Track | null }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query Track($id: String!) { track(id: $id) { ...TrackFields } }
    `, { id });
    return data.track;
  }

  async likedTracks(): Promise<Track[]> {
    const data = await this.http.execute<{ likedTracks: Track[] }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      query LikedTracks { likedTracks { ...TrackFields } }
    `);
    return data.likedTracks;
  }

  async likeTrack(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation LikeTrack($id: String!) { likeTrack(id: $id) }
    `, { id });
  }

  async unlikeTrack(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }
    `, { id });
  }

  // ---------------------------------------------------------------------------
  // Search
  // ---------------------------------------------------------------------------

  async search(term: string): Promise<SearchResults> {
    const data = await this.http.execute<{ search: SearchResults }>(/* GraphQL */ `
      ${TRACK_FIELDS}
      ${ALBUM_FIELDS}
      ${ARTIST_FIELDS}
      query Search($term: String!) {
        search(term: $term) {
          artists { ...ArtistFields }
          albums { ...AlbumFields }
          tracks { ...TrackFields }
          likedTracks { ...TrackFields }
          likedAlbums { ...AlbumFields }
        }
      }
    `, { term });
    return data.search;
  }

  // ---------------------------------------------------------------------------
  // Library management
  // ---------------------------------------------------------------------------

  async scan(): Promise<void> {
    await this.http.execute(/* GraphQL */ `mutation ScanLibrary { scanLibrary }`);
  }
}

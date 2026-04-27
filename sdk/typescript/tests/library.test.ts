import { describe, it, expect, vi } from 'vitest';
import { LibraryApi } from '../src/api/library.js';
import type { HttpTransport } from '../src/transport.js';

function makeTransport(result: unknown) {
  return { execute: vi.fn().mockResolvedValue(result) } as unknown as HttpTransport;
}

const ALBUM = {
  id: 'a1',
  title: 'The Wall',
  artist: 'Pink Floyd',
  year: 1979,
  yearString: '1979',
  albumArt: null,
  md5: 'abc',
  artistId: 'ar1',
  tracks: [],
};

const TRACK = {
  id: 't1',
  title: 'Hey You',
  artist: 'Pink Floyd',
  album: 'The Wall',
  genre: 'Rock',
  disc: '2',
  trackString: '5',
  yearString: '1979',
  composer: 'Roger Waters',
  comment: '',
  albumArtist: 'Pink Floyd',
  grouping: '',
  discnum: 2,
  tracknum: 5,
  layer: 3,
  year: 1979,
  bitrate: 320,
  frequency: 44100,
  filesize: 8888888,
  length: 270000,
  elapsed: 0,
  path: '/Music/Pink Floyd/The Wall/CD2/05 Hey You.mp3',
};

describe('LibraryApi', () => {
  it('returns a list of albums', async () => {
    const api = new LibraryApi(makeTransport({ albums: [ALBUM] }));
    const albums = await api.albums();
    expect(albums).toHaveLength(1);
    expect(albums[0]!.title).toBe('The Wall');
  });

  it('returns null for unknown album id', async () => {
    const api = new LibraryApi(makeTransport({ album: null }));
    expect(await api.album('unknown')).toBeNull();
  });

  it('returns search results', async () => {
    const api = new LibraryApi(
      makeTransport({
        search: {
          artists: [],
          albums: [ALBUM],
          tracks: [TRACK],
          likedTracks: [],
          likedAlbums: [],
        },
      }),
    );
    const results = await api.search('wall');
    expect(results.albums).toHaveLength(1);
    expect(results.tracks).toHaveLength(1);
  });

  it('calls likeTrack mutation with the correct id', async () => {
    const transport = makeTransport({});
    const api = new LibraryApi(transport);
    await api.likeTrack('track-99');
    expect(transport.execute).toHaveBeenCalledWith(
      expect.stringContaining('likeTrack'),
      expect.objectContaining({ id: 'track-99' }),
    );
  });
});

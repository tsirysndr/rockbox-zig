import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PlaybackApi } from '../src/api/playback.js';
import { PlaybackStatus } from '../src/types.js';
import type { HttpTransport } from '../src/transport.js';

function makeTransport(result: unknown) {
  return { execute: vi.fn().mockResolvedValue(result) } as unknown as HttpTransport;
}

const TRACK = {
  id: 'track-1',
  title: 'Comfortably Numb',
  artist: 'Pink Floyd',
  album: 'The Wall',
  genre: 'Rock',
  disc: '2',
  trackString: '27',
  yearString: '1979',
  composer: 'Roger Waters',
  comment: '',
  albumArtist: 'Pink Floyd',
  grouping: '',
  discnum: 2,
  tracknum: 27,
  layer: 3,
  year: 1979,
  bitrate: 320,
  frequency: 44100,
  filesize: 12345678,
  length: 382000,
  elapsed: 0,
  path: '/Music/Pink Floyd/The Wall/CD2/27 Comfortably Numb.mp3',
};

describe('PlaybackApi', () => {
  describe('status', () => {
    it('returns PlaybackStatus.Playing when firmware returns 1', async () => {
      const api = new PlaybackApi(makeTransport({ status: 1 }));
      expect(await api.status()).toBe(PlaybackStatus.Playing);
    });

    it('returns PlaybackStatus.Stopped when firmware returns 0', async () => {
      const api = new PlaybackApi(makeTransport({ status: 0 }));
      expect(await api.status()).toBe(PlaybackStatus.Stopped);
    });
  });

  describe('currentTrack', () => {
    it('returns the track when one is playing', async () => {
      const api = new PlaybackApi(makeTransport({ currentTrack: TRACK }));
      const track = await api.currentTrack();
      expect(track?.title).toBe('Comfortably Numb');
      expect(track?.length).toBe(382000);
    });

    it('returns null when nothing is playing', async () => {
      const api = new PlaybackApi(makeTransport({ currentTrack: null }));
      expect(await api.currentTrack()).toBeNull();
    });
  });

  describe('transport controls', () => {
    it('calls pause mutation', async () => {
      const transport = makeTransport({});
      const api = new PlaybackApi(transport);
      await api.pause();
      expect(transport.execute).toHaveBeenCalledWith(expect.stringContaining('pause'));
    });

    it('calls seek with the given position', async () => {
      const transport = makeTransport({});
      const api = new PlaybackApi(transport);
      await api.seek(60_000);
      expect(transport.execute).toHaveBeenCalledWith(
        expect.stringContaining('fastForwardRewind'),
        expect.objectContaining({ newTime: 60_000 }),
      );
    });
  });

  describe('play helpers', () => {
    it('calls playAlbum with shuffle option', async () => {
      const transport = makeTransport({});
      const api = new PlaybackApi(transport);
      await api.playAlbum('album-42', { shuffle: true });
      expect(transport.execute).toHaveBeenCalledWith(
        expect.stringContaining('playAlbum'),
        expect.objectContaining({ albumId: 'album-42', shuffle: true }),
      );
    });

    it('calls playDirectory with recurse option', async () => {
      const transport = makeTransport({});
      const api = new PlaybackApi(transport);
      await api.playDirectory('/Music/Jazz', { recurse: true, shuffle: true });
      expect(transport.execute).toHaveBeenCalledWith(
        expect.stringContaining('playDirectory'),
        expect.objectContaining({ path: '/Music/Jazz', recurse: true }),
      );
    });
  });
});

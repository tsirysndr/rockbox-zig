const { ops } = Deno.core;

const library = {
  album: {
    getAlbum: (id) => ops.op_get_album(id),
    getAlbums: () => ops.op_get_albums(),
  },
  artist: {
    getArtist: (id) => ops.op_get_artist(id),
    getArtists: () => ops.op_get_artists(),
  },
  track: {
    getTrack: (id) => ops.op_get_track(id),
    getTracks: () => ops.op_get_tracks(),
  },
};

globalThis.rb = { ...globalThis.rb, library };

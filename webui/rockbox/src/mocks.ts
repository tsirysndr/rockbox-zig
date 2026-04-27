import {
  nextTracks,
  previousTracks,
} from "./Components/ControlBar/PlayQueue/mocks";

export const mockCurrentTrack = {
  currentTrack: {
    id: "cm272oeon00esm9634q1lw5ae",
    title: "Set It Off",
    artist: "Boosie Badazz",
    album: "Bad Azz",
    albumArt:
      "https://resources.tidal.com/images/31ce8fc7/b10c/47ee/991d/6fab3e15dbe4/320x320.jpg",
    artistId: "cm272ocoi003km963fy45i7cn",
    albumId: "cm272ocoi003lm963xo1d7wb2",
    elapsed: 153762,
    length: 284633,
    year: 2006,
    yearString: "2006-09-19",
  },
};

export const mockPlaybackStatus = { status: 1 };

export const mockCurrentPlaylist = {
  playlistGetCurrent: {
    index: 2,
    amount: previousTracks.length + nextTracks.length,
    maxPlaylistSize: 10000,
    tracks: [...previousTracks, ...nextTracks].map((x) => ({
      id: x.id,
      title: x.title,
      artist: x.artist,
      albumArt: x.cover,
      artistId: null,
      albumId: null,
    })),
  },
};

export const mockTracks = {
  tracks: [
    {
      id: "cm272oeon00esm9634q1lw5ae",
      tracknum: 1,
      title: "Set It Off",
      artist: "Boosie Badazz",
      album: "Bad Azz",
      discnum: 1,
      albumArtist: "Boosie Badazz",
      artistId: "cm272ocoi003km963fy45i7cn",
      albumId: "cm272ocoi003lm963xo1d7wb2",
      albumArt:
        "https://resources.tidal.com/images/31ce8fc7/b10c/47ee/991d/6fab3e15dbe4/320x320.jpg",
    },
  ],
};

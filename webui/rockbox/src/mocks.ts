import {
  GET_CURRENT_TRACK,
  GET_PLAYBACK_STATUS,
} from "./GraphQL/Playback/Query";

export const mocks = [
  {
    request: {
      query: GET_CURRENT_TRACK,
    },
    result: {
      data: {
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
      },
    },
  },
  {
    request: {
      query: GET_PLAYBACK_STATUS,
    },
    result: { data: { status: 1 } },
  },
];

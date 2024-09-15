const playback = {
  play: () => {},
  pause: () => {},
  resume: () => {},
  next: () => {},
  previous: () => {},
  fastForwardRewind: () => {},
  status: () => {},
  currentTrack: () => {},
  flushAndReload: () => {},
  getFilePosition: () => {},
  hardStop: () => {},
};

globalThis.rb = { ...globalThis.rb, playback };

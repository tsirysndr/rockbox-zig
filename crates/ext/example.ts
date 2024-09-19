console.log(rb);
console.log(await rb.system.getRockboxVersion());
console.log(await rb.system.getGlobalStatus());
console.log(await rb.settings.getGlobalSettings());
console.log(await rb.playlist.playlistResume());
console.log(await rb.playlist.resumeTrack());
console.log(await rb.playlist.getCurrent());

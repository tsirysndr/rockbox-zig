console.log(rb);
console.log(await rb.system.getRockboxVersion());
console.log(await rb.system.getGlobalStatus());
console.log(await rb.settings.getGlobalSettings());
console.log(await rb.playlist.playlistResume());
console.log(await rb.playlist.resumeTrack());
console.log(await rb.playlist.getCurrent());
console.log(await rb.playlist.amount());
console.log(await rb.browse.tree.getEntries("/"));
console.log(await rb.library.album.getAlbums());
console.log(await rb.library.artist.getArtists());
console.log(await rb.library.track.getTracks());
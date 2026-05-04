import { FC, useMemo } from "react";
import {
  useGetAlbumsQuery,
  useGetArtistsQuery,
  useGetSavedPlaylistsQuery,
  useGetSmartPlaylistsQuery,
} from "../../Hooks/GraphQL";
import Home, { HomeAlbum, HomeArtist, HomePlaylist } from "./Home";

const HomeWithData: FC = () => {
  const { data: albumsData, isLoading: albumsLoading } = useGetAlbumsQuery();
  const { data: artistsData, isLoading: artistsLoading } = useGetArtistsQuery();
  const { data: savedData } = useGetSavedPlaylistsQuery();
  const { data: smartData } = useGetSmartPlaylistsQuery();

  // Track count is computed from `album.tracks` (the albums query already
  // returns the full track list). It's a weak proxy for popularity until real
  // play_count stats are exposed via gRPC.
  const albums = useMemo(
    () =>
      (albumsData?.albums ?? []).map((a) => ({
        id: a.id,
        title: a.title,
        artist: a.artist,
        artistId: a.artistId,
        year: a.year,
        cover: a.albumArt
          ? `${location.protocol}//${location.host}/covers/${a.albumArt}`
          : undefined,
        trackCount: a.tracks?.length ?? 0,
      })),
    [albumsData],
  );

  const artists: HomeArtist[] = useMemo(
    () =>
      (artistsData?.artists ?? []).map((a) => ({
        id: a.id,
        name: a.name,
        image: a.image
          ? a.image.startsWith("http")
            ? a.image
            : `${location.protocol}//${location.host}/covers/${a.image}`
          : null,
      })),
    [artistsData],
  );

  const saved: HomePlaylist[] = useMemo(
    () =>
      (savedData?.savedPlaylists ?? []).map((p) => ({
        id: p.id,
        name: p.name,
        description: p.description ?? null,
        image: p.image
          ? p.image.startsWith("http")
            ? p.image
            : `${location.protocol}//${location.host}/covers/${p.image}`
          : null,
        isSmart: false,
      })),
    [savedData],
  );

  const smart: HomePlaylist[] = useMemo(
    () =>
      (smartData?.smartPlaylists ?? []).map((p) => ({
        id: p.id,
        name: p.name,
        description: p.description ?? null,
        image: p.image
          ? p.image.startsWith("http")
            ? p.image
            : `${location.protocol}//${location.host}/covers/${p.image}`
          : null,
        isSmart: true,
      })),
    [smartData],
  );

  // Quick picks: prefer system smart playlists first, then saved.
  const quickPicks = useMemo(
    () => [...smart, ...saved].slice(0, 6),
    [smart, saved],
  );

  // Recently played → cuid is time-ordered, so sorting by id DESC approximates
  // date_added DESC (newest album in the library first).
  const recentlyPlayed: HomeAlbum[] = useMemo(
    () =>
      [...albums]
        .sort((a, b) => (a.id < b.id ? 1 : a.id > b.id ? -1 : 0))
        .slice(0, 8),
    [albums],
  );

  // Popular albums → most tracks first (proxy for popularity).
  const popularAlbums: HomeAlbum[] = useMemo(
    () =>
      [...albums]
        .sort((a, b) => b.trackCount - a.trackCount || a.title.localeCompare(b.title))
        .slice(0, 12),
    [albums],
  );

  const madeForYou = smart.slice(0, 12);
  const topArtists = artists.slice(0, 12);

  return (
    <Home
      recentlyPlayed={recentlyPlayed}
      topArtists={topArtists}
      popularAlbums={popularAlbums}
      madeForYou={madeForYou}
      quickPicks={quickPicks}
      loading={albumsLoading || artistsLoading}
    />
  );
};

export default HomeWithData;

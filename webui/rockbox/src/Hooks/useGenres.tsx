import {
  useMutation,
  UseMutationOptions,
  useQuery,
  UseQueryOptions,
} from "@tanstack/react-query";
import { graphqlClient } from "../lib/graphql-client";

// ── Types ────────────────────────────────────────────────────────────────────

export type GenreSummary = {
  id: string;
  name: string;
  description?: string | null;
  image?: string | null;
  trackCount: number;
};

export type GenreTrack = {
  id?: string | null;
  title: string;
  artist: string;
  album: string;
  albumArt?: string | null;
  albumArtist: string;
  artistId?: string | null;
  albumId?: string | null;
  path: string;
  length: number;
};

export type GenreAlbum = {
  id: string;
  title: string;
  artist: string;
  albumArt?: string | null;
  year: number;
  yearString: string;
  artistId: string;
};

export type GenreArtist = {
  id: string;
  name: string;
  image?: string | null;
};

export type GenreDetail = {
  id: string;
  name: string;
  description?: string | null;
  image?: string | null;
  trackCount: number;
  tracks: GenreTrack[];
  albums: GenreAlbum[];
  artists: GenreArtist[];
};

// ── Documents ────────────────────────────────────────────────────────────────

const GET_GENRES = /* GraphQL */ `
  query GetGenres {
    genres {
      id
      name
      description
      image
      trackCount
    }
  }
`;

const GET_GENRE = /* GraphQL */ `
  query GetGenre($id: String!) {
    genre(id: $id) {
      id
      name
      description
      image
      trackCount
      tracks {
        id
        title
        artist
        album
        albumArt
        albumArtist
        artistId
        albumId
        path
        length
      }
      albums {
        id
        title
        artist
        albumArt
        year
        yearString
        artistId
      }
      artists {
        id
        name
        image
      }
    }
  }
`;

const PLAY_GENRE_TRACKS = /* GraphQL */ `
  mutation PlayGenreTracks(
    $genreId: String!
    $shuffle: Boolean
    $position: Int
  ) {
    playGenreTracks(genreId: $genreId, shuffle: $shuffle, position: $position)
  }
`;

// ── Hooks ────────────────────────────────────────────────────────────────────

export function useGetGenresQuery(
  options?: Omit<
    UseQueryOptions<{ genres: GenreSummary[] }>,
    "queryKey" | "queryFn"
  >,
) {
  return useQuery<{ genres: GenreSummary[] }>({
    queryKey: ["GetGenres"],
    queryFn: () => graphqlClient.request<{ genres: GenreSummary[] }>(GET_GENRES),
    ...options,
  });
}

export function useGetGenreQuery(
  variables: { id: string },
  options?: Omit<
    UseQueryOptions<{ genre: GenreDetail | null }>,
    "queryKey" | "queryFn"
  >,
) {
  return useQuery<{ genre: GenreDetail | null }>({
    queryKey: ["GetGenre", variables],
    queryFn: () =>
      graphqlClient.request<{ genre: GenreDetail | null }>(GET_GENRE, variables),
    ...options,
  });
}

type PlayGenreVars = {
  genreId: string;
  shuffle?: boolean;
  position?: number;
};

export function usePlayGenreTracksMutation(
  options?: UseMutationOptions<
    { playGenreTracks: number },
    unknown,
    PlayGenreVars
  >,
) {
  return useMutation<
    { playGenreTracks: number },
    unknown,
    PlayGenreVars
  >({
    mutationKey: ["PlayGenreTracks"],
    mutationFn: (variables) =>
      graphqlClient.request<{ playGenreTracks: number }>(
        PLAY_GENRE_TRACKS,
        variables as Record<string, unknown>,
      ),
    ...options,
  });
}

import { gql } from "@apollo/client";

export const GET_ALBUMS = gql`
  query GetAlbums {
    albums {
      id
      title
      artist
      albumArt
      year
      yearString
      artistId
      md5
      tracks {
        id
        title
        artist
        album
        albumArtist
        artistId
        albumId
        path
        length
      }
    }
  }
`;

export const GET_ARTISTS = gql`
  query GetArtists {
    artists {
      id
      name
    }
  }
`;

export const GET_ARTIST = gql`
  query GetArtist($id: String!) {
    artist(id: $id) {
      id
      name
      albums {
        id
        title
        artist
        albumArt
        year
        yearString
        artistId
        md5
      }
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
    }
  }
`;

export const GET_TRACKS = gql`
  query Tracks {
    tracks {
      id
      tracknum
      title
      artist
      album
      discnum
      albumArtist
      artistId
      albumId
      albumArt
      path
      length
    }
  }
`;

export const GET_ALBUM = gql`
  query GetAlbum($id: String!) {
    album(id: $id) {
      id
      title
      artist
      albumArt
      year
      yearString
      artistId
      md5
      tracks {
        id
        title
        tracknum
        artist
        album
        discnum
        albumArtist
        artistId
        albumId
        path
        length
      }
    }
  }
`;

export const GET_LIKED_TRACKS = gql`
  query GetLikedTracks {
    likedTracks {
      id
      tracknum
      title
      artist
      album
      discnum
      albumArtist
      artistId
      albumId
      albumArt
      path
      length
    }
  }
`;

export const GET_LIKED_ALBUMS = gql`
  query GetLikedAlbums {
    likedAlbums {
      id
      title
      artist
      albumArt
      year
      yearString
      artistId
      md5
      tracks {
        id
        title
        artist
        album
        albumArtist
        artistId
        albumId
        path
        length
      }
    }
  }
`;

export const SEARCH = gql`
  query Search($term: String!) {
    search(term: $term) {
      tracks {
        id
        title
        artist
        album
        albumArtist
        path
        albumArt
        length
        composer
        comment
        albumId
        artistId
      }
      albums {
        id
        title
        year
        yearString
        albumArt
        artist
        artistId
      }
      artists {
        id
        name
        image
      }
      likedTracks {
        id
        title
        artist
        album
        albumArtist
        path
        albumArt
        length
        composer
        comment
        albumId
        artistId
      }
      likedAlbums {
        id
        title
        albumArt
        artist
        artistId
        year
      }
    }
  }
`;

//// Browse and search the indexed music library.
////
//// ```gleam
//// let assert Ok(albums) = library.albums(client)
//// let assert Ok(results) = library.search(client, "miles davis")
//// ```

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/types.{
  type Album, type Artist, type SearchResults, type Track,
}

const track_fields = "
  fragment TrackFields on Track {
    id title artist album genre disc trackString yearString
    composer comment albumArtist grouping
    discnum tracknum layer year bitrate frequency
    filesize length elapsed path
    albumId artistId genreId albumArt
  }
"

const album_fields = "
  fragment AlbumFields on Album {
    id title artist year yearString albumArt md5 artistId copyrightMessage
  }
"

const artist_fields = "
  fragment ArtistFields on Artist {
    id name bio image
  }
"

// ---------------------------------------------------------------------------
// Albums
// ---------------------------------------------------------------------------

pub fn albums(client: Client) -> Result(List(Album), Error) {
  let decoder = {
    use albums <- decode.field("albums", decode.list(types.album_decoder()))
    decode.success(albums)
  }
  let q = album_fields <> "
    query Albums {
      albums { ...AlbumFields tracks { id title path length albumArt } }
    }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

pub fn album(client: Client, id: String) -> Result(Option(Album), Error) {
  let decoder = {
    use album <- decode.field(
      "album",
      decode.optional(types.album_decoder()),
    )
    decode.success(album)
  }
  let q = track_fields <> album_fields <> "
    query Album($id: String!) {
      album(id: $id) { ...AlbumFields tracks { ...TrackFields } }
    }
  "
  rockbox.query(client, q, json.object([#("id", json.string(id))]), decoder)
}

pub fn liked_albums(client: Client) -> Result(List(Album), Error) {
  let decoder = {
    use albums <- decode.field(
      "likedAlbums",
      decode.list(types.album_decoder()),
    )
    decode.success(albums)
  }
  let q = album_fields <> "
    query LikedAlbums { likedAlbums { ...AlbumFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

pub fn like_album(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation LikeAlbum($id: String!) { likeAlbum(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

pub fn unlike_album(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation UnlikeAlbum($id: String!) { unlikeAlbum(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

// ---------------------------------------------------------------------------
// Artists
// ---------------------------------------------------------------------------

pub fn artists(client: Client) -> Result(List(Artist), Error) {
  let decoder = {
    use artists <- decode.field("artists", decode.list(types.artist_decoder()))
    decode.success(artists)
  }
  let q = artist_fields <> "
    query Artists {
      artists { ...ArtistFields albums { id title albumArt year } }
    }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

pub fn artist(client: Client, id: String) -> Result(Option(Artist), Error) {
  let decoder = {
    use artist <- decode.field(
      "artist",
      decode.optional(types.artist_decoder()),
    )
    decode.success(artist)
  }
  let q = artist_fields <> track_fields <> "
    query Artist($id: String!) {
      artist(id: $id) {
        ...ArtistFields
        albums {
          id title albumArt year yearString md5 artistId
          tracks { id title path length }
        }
        tracks { ...TrackFields }
      }
    }
  "
  rockbox.query(client, q, json.object([#("id", json.string(id))]), decoder)
}

// ---------------------------------------------------------------------------
// Tracks
// ---------------------------------------------------------------------------

pub fn tracks(client: Client) -> Result(List(Track), Error) {
  let decoder = {
    use tracks <- decode.field("tracks", decode.list(types.track_decoder()))
    decode.success(tracks)
  }
  let q = track_fields <> "
    query Tracks { tracks { ...TrackFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

pub fn track(client: Client, id: String) -> Result(Option(Track), Error) {
  let decoder = {
    use track <- decode.field(
      "track",
      decode.optional(types.track_decoder()),
    )
    decode.success(track)
  }
  let q = track_fields <> "
    query Track($id: String!) { track(id: $id) { ...TrackFields } }
  "
  rockbox.query(client, q, json.object([#("id", json.string(id))]), decoder)
}

pub fn liked_tracks(client: Client) -> Result(List(Track), Error) {
  let decoder = {
    use tracks <- decode.field(
      "likedTracks",
      decode.list(types.track_decoder()),
    )
    decode.success(tracks)
  }
  let q = track_fields <> "
    query LikedTracks { likedTracks { ...TrackFields } }
  "
  rockbox.query(client, q, json.object([]), decoder)
}

pub fn like_track(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation LikeTrack($id: String!) { likeTrack(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

pub fn unlike_track(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation UnlikeTrack($id: String!) { unlikeTrack(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

pub fn search(client: Client, term: String) -> Result(SearchResults, Error) {
  let decoder = {
    use results <- decode.field("search", types.search_results_decoder())
    decode.success(results)
  }
  let q = track_fields <> album_fields <> artist_fields <> "
    query Search($term: String!) {
      search(term: $term) {
        artists { ...ArtistFields }
        albums { ...AlbumFields }
        tracks { ...TrackFields }
        likedTracks { ...TrackFields }
        likedAlbums { ...AlbumFields }
      }
    }
  "
  rockbox.query(client, q, json.object([#("term", json.string(term))]), decoder)
}

// ---------------------------------------------------------------------------
// Library management
// ---------------------------------------------------------------------------

pub fn scan(client: Client) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation ScanLibrary { scanLibrary }",
    json.object([]),
  )
}

//// Persisted playlists and their folders.
////
//// Builders make optional fields ergonomic:
////
//// ```gleam
//// let input =
////   saved_playlists.new("Workout Mix")
////   |> saved_playlists.with_description("Tracks I run to")
////   |> saved_playlists.with_tracks(["track-id-1", "track-id-2"])
////
//// let assert Ok(playlist) = saved_playlists.create(client, input)
//// ```

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option, None, Some}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type SavedPlaylist, type SavedPlaylistFolder}

// ---------------------------------------------------------------------------
// CreateInput builder
// ---------------------------------------------------------------------------

pub opaque type CreateInput {
  CreateInput(
    name: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
    track_ids: Option(List(String)),
  )
}

/// Start a new create-input with just a name.
pub fn new(name: String) -> CreateInput {
  CreateInput(
    name: name,
    description: None,
    image: None,
    folder_id: None,
    track_ids: None,
  )
}

pub fn with_description(input: CreateInput, value: String) -> CreateInput {
  CreateInput(..input, description: Some(value))
}

pub fn with_image(input: CreateInput, value: String) -> CreateInput {
  CreateInput(..input, image: Some(value))
}

pub fn with_folder(input: CreateInput, folder_id: String) -> CreateInput {
  CreateInput(..input, folder_id: Some(folder_id))
}

pub fn with_tracks(input: CreateInput, track_ids: List(String)) -> CreateInput {
  CreateInput(..input, track_ids: Some(track_ids))
}

// ---------------------------------------------------------------------------
// UpdateInput builder
// ---------------------------------------------------------------------------

pub opaque type UpdateInput {
  UpdateInput(
    name: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
  )
}

/// Build an update payload — `name` is required by the GraphQL schema even
/// when only changing other fields, so it lives in the constructor.
pub fn update(name: String) -> UpdateInput {
  UpdateInput(name: name, description: None, image: None, folder_id: None)
}

pub fn update_description(input: UpdateInput, value: String) -> UpdateInput {
  UpdateInput(..input, description: Some(value))
}

pub fn update_image(input: UpdateInput, value: String) -> UpdateInput {
  UpdateInput(..input, image: Some(value))
}

pub fn update_folder(input: UpdateInput, folder_id: String) -> UpdateInput {
  UpdateInput(..input, folder_id: Some(folder_id))
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// List all saved playlists, optionally scoped to a single folder.
pub fn list(
  client: Client,
  folder_id: Option(String),
) -> Result(List(SavedPlaylist), Error) {
  let decoder = {
    use playlists <- decode.field(
      "savedPlaylists",
      decode.list(types.saved_playlist_decoder()),
    )
    decode.success(playlists)
  }
  let vars =
    transport.variables([#("folderId", option.map(folder_id, json.string))])
  rockbox.query(
    client,
    "query SavedPlaylists($folderId: String) {
       savedPlaylists(folderId: $folderId) {
         id name description image folderId trackCount createdAt updatedAt
       }
     }",
    vars,
    decoder,
  )
}

pub fn get(
  client: Client,
  id: String,
) -> Result(Option(SavedPlaylist), Error) {
  let decoder = {
    use playlist <- decode.field(
      "savedPlaylist",
      decode.optional(types.saved_playlist_decoder()),
    )
    decode.success(playlist)
  }
  rockbox.query(
    client,
    "query SavedPlaylist($id: String!) {
       savedPlaylist(id: $id) {
         id name description image folderId trackCount createdAt updatedAt
       }
     }",
    json.object([#("id", json.string(id))]),
    decoder,
  )
}

pub fn track_ids(
  client: Client,
  playlist_id: String,
) -> Result(List(String), Error) {
  let decoder = {
    use ids <- decode.field(
      "savedPlaylistTrackIds",
      decode.list(decode.string),
    )
    decode.success(ids)
  }
  rockbox.query(
    client,
    "query SavedPlaylistTrackIds($playlistId: String!) {
       savedPlaylistTrackIds(playlistId: $playlistId)
     }",
    json.object([#("playlistId", json.string(playlist_id))]),
    decoder,
  )
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

pub fn create(
  client: Client,
  input: CreateInput,
) -> Result(SavedPlaylist, Error) {
  let vars =
    transport.variables([
      #("name", Some(json.string(input.name))),
      #("description", option.map(input.description, json.string)),
      #("image", option.map(input.image, json.string)),
      #("folderId", option.map(input.folder_id, json.string)),
      #(
        "trackIds",
        option.map(input.track_ids, fn(ids) { json.array(ids, json.string) }),
      ),
    ])
  let decoder = {
    use playlist <- decode.field(
      "createSavedPlaylist",
      types.saved_playlist_decoder(),
    )
    decode.success(playlist)
  }
  rockbox.query(
    client,
    "mutation CreateSavedPlaylist(
       $name: String!, $description: String, $image: String,
       $folderId: String, $trackIds: [String!]
     ) {
       createSavedPlaylist(
         name: $name, description: $description, image: $image,
         folderId: $folderId, trackIds: $trackIds
       ) {
         id name description image folderId trackCount createdAt updatedAt
       }
     }",
    vars,
    decoder,
  )
}

pub fn save(
  client: Client,
  id: String,
  input: UpdateInput,
) -> Result(Nil, Error) {
  let vars =
    transport.variables([
      #("id", Some(json.string(id))),
      #("name", Some(json.string(input.name))),
      #("description", option.map(input.description, json.string)),
      #("image", option.map(input.image, json.string)),
      #("folderId", option.map(input.folder_id, json.string)),
    ])
  rockbox.execute(
    client,
    "mutation UpdateSavedPlaylist(
       $id: String!, $name: String!, $description: String, $image: String, $folderId: String
     ) {
       updateSavedPlaylist(
         id: $id, name: $name, description: $description, image: $image, folderId: $folderId
       )
     }",
    vars,
  )
}

pub fn delete(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation DeleteSavedPlaylist($id: String!) { deleteSavedPlaylist(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

pub fn add_tracks(
  client: Client,
  playlist_id: String,
  track_ids: List(String),
) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation AddTracksToSavedPlaylist($playlistId: String!, $trackIds: [String!]!) {
       addTracksToSavedPlaylist(playlistId: $playlistId, trackIds: $trackIds)
     }",
    json.object([
      #("playlistId", json.string(playlist_id)),
      #("trackIds", json.array(track_ids, json.string)),
    ]),
  )
}

pub fn remove_track(
  client: Client,
  playlist_id: String,
  track_id: String,
) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation RemoveTrackFromSavedPlaylist($playlistId: String!, $trackId: String!) {
       removeTrackFromSavedPlaylist(playlistId: $playlistId, trackId: $trackId)
     }",
    json.object([
      #("playlistId", json.string(playlist_id)),
      #("trackId", json.string(track_id)),
    ]),
  )
}

pub fn play(client: Client, playlist_id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation PlaySavedPlaylist($playlistId: String!) { playSavedPlaylist(playlistId: $playlistId) }",
    json.object([#("playlistId", json.string(playlist_id))]),
  )
}

// ---------------------------------------------------------------------------
// Folders
// ---------------------------------------------------------------------------

pub fn folders(client: Client) -> Result(List(SavedPlaylistFolder), Error) {
  let decoder = {
    use folders <- decode.field(
      "playlistFolders",
      decode.list(types.saved_playlist_folder_decoder()),
    )
    decode.success(folders)
  }
  rockbox.query(
    client,
    "query PlaylistFolders { playlistFolders { id name createdAt updatedAt } }",
    json.object([]),
    decoder,
  )
}

pub fn create_folder(
  client: Client,
  name: String,
) -> Result(SavedPlaylistFolder, Error) {
  let decoder = {
    use folder <- decode.field(
      "createPlaylistFolder",
      types.saved_playlist_folder_decoder(),
    )
    decode.success(folder)
  }
  rockbox.query(
    client,
    "mutation CreatePlaylistFolder($name: String!) {
       createPlaylistFolder(name: $name) { id name createdAt updatedAt }
     }",
    json.object([#("name", json.string(name))]),
    decoder,
  )
}

pub fn delete_folder(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation DeletePlaylistFolder($id: String!) { deletePlaylistFolder(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

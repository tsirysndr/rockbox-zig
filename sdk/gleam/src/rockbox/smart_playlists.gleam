//// Rule-based "smart" playlists that auto-update from listening stats.

import gleam/dynamic/decode
import gleam/json
import gleam/option.{type Option, None, Some}
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/internal/transport
import rockbox/types.{type SmartPlaylist, type TrackStats}

// ---------------------------------------------------------------------------
// Input builders
// ---------------------------------------------------------------------------

pub opaque type CreateInput {
  CreateInput(
    name: String,
    rules: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
  )
}

/// `rules` is the JSON-encoded smart-playlist rule set. The format matches
/// the rockboxd schema — see the project README for details.
pub fn new(name: String, rules: String) -> CreateInput {
  CreateInput(
    name: name,
    rules: rules,
    description: None,
    image: None,
    folder_id: None,
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

pub opaque type UpdateInput {
  UpdateInput(
    name: String,
    rules: String,
    description: Option(String),
    image: Option(String),
    folder_id: Option(String),
  )
}

pub fn update(name: String, rules: String) -> UpdateInput {
  UpdateInput(
    name: name,
    rules: rules,
    description: None,
    image: None,
    folder_id: None,
  )
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

pub fn list(client: Client) -> Result(List(SmartPlaylist), Error) {
  let decoder = {
    use playlists <- decode.field(
      "smartPlaylists",
      decode.list(types.smart_playlist_decoder()),
    )
    decode.success(playlists)
  }
  rockbox.query(
    client,
    "query SmartPlaylists {
       smartPlaylists {
         id name description image folderId isSystem rules createdAt updatedAt
       }
     }",
    json.object([]),
    decoder,
  )
}

pub fn get(client: Client, id: String) -> Result(Option(SmartPlaylist), Error) {
  let decoder = {
    use playlist <- decode.field(
      "smartPlaylist",
      decode.optional(types.smart_playlist_decoder()),
    )
    decode.success(playlist)
  }
  rockbox.query(
    client,
    "query SmartPlaylist($id: String!) {
       smartPlaylist(id: $id) {
         id name description image folderId isSystem rules createdAt updatedAt
       }
     }",
    json.object([#("id", json.string(id))]),
    decoder,
  )
}

pub fn track_ids(client: Client, id: String) -> Result(List(String), Error) {
  let decoder = {
    use ids <- decode.field("smartPlaylistTrackIds", decode.list(decode.string))
    decode.success(ids)
  }
  rockbox.query(
    client,
    "query SmartPlaylistTrackIds($id: String!) { smartPlaylistTrackIds(id: $id) }",
    json.object([#("id", json.string(id))]),
    decoder,
  )
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

pub fn create(
  client: Client,
  input: CreateInput,
) -> Result(SmartPlaylist, Error) {
  let vars =
    transport.variables([
      #("name", Some(json.string(input.name))),
      #("rules", Some(json.string(input.rules))),
      #("description", option.map(input.description, json.string)),
      #("image", option.map(input.image, json.string)),
      #("folderId", option.map(input.folder_id, json.string)),
    ])
  let decoder = {
    use playlist <- decode.field(
      "createSmartPlaylist",
      types.smart_playlist_decoder(),
    )
    decode.success(playlist)
  }
  rockbox.query(
    client,
    "mutation CreateSmartPlaylist(
       $name: String!, $rules: String!, $description: String,
       $image: String, $folderId: String
     ) {
       createSmartPlaylist(
         name: $name, rules: $rules, description: $description,
         image: $image, folderId: $folderId
       ) {
         id name description image folderId isSystem rules createdAt updatedAt
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
      #("rules", Some(json.string(input.rules))),
      #("description", option.map(input.description, json.string)),
      #("image", option.map(input.image, json.string)),
      #("folderId", option.map(input.folder_id, json.string)),
    ])
  rockbox.execute(
    client,
    "mutation UpdateSmartPlaylist(
       $id: String!, $name: String!, $rules: String!,
       $description: String, $image: String, $folderId: String
     ) {
       updateSmartPlaylist(
         id: $id, name: $name, rules: $rules, description: $description,
         image: $image, folderId: $folderId
       )
     }",
    vars,
  )
}

pub fn delete(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation DeleteSmartPlaylist($id: String!) { deleteSmartPlaylist(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

pub fn play(client: Client, id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation PlaySmartPlaylist($id: String!) { playSmartPlaylist(id: $id) }",
    json.object([#("id", json.string(id))]),
  )
}

// ---------------------------------------------------------------------------
// Listening stats — feed smart playlist rules
// ---------------------------------------------------------------------------

pub fn track_stats(
  client: Client,
  track_id: String,
) -> Result(Option(TrackStats), Error) {
  let decoder = {
    use stats <- decode.field(
      "trackStats",
      decode.optional(types.track_stats_decoder()),
    )
    decode.success(stats)
  }
  rockbox.query(
    client,
    "query TrackStats($trackId: String!) {
       trackStats(trackId: $trackId) {
         trackId playCount skipCount lastPlayed lastSkipped updatedAt
       }
     }",
    json.object([#("trackId", json.string(track_id))]),
    decoder,
  )
}

pub fn record_played(client: Client, track_id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation RecordTrackPlayed($trackId: String!) { recordTrackPlayed(trackId: $trackId) }",
    json.object([#("trackId", json.string(track_id))]),
  )
}

pub fn record_skipped(client: Client, track_id: String) -> Result(Nil, Error) {
  rockbox.execute(
    client,
    "mutation RecordTrackSkipped($trackId: String!) { recordTrackSkipped(trackId: $trackId) }",
    json.object([#("trackId", json.string(track_id))]),
  )
}

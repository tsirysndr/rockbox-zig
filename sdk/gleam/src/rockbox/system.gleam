//// Daemon information — version string and global runtime status.

import gleam/dynamic/decode
import gleam/json
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/types.{type SystemStatus}

pub fn version(client: Client) -> Result(String, Error) {
  let decoder = {
    use v <- decode.field("rockboxVersion", decode.string)
    decode.success(v)
  }
  rockbox.query(
    client,
    "query Version { rockboxVersion }",
    json.object([]),
    decoder,
  )
}

pub fn status(client: Client) -> Result(SystemStatus, Error) {
  let decoder = {
    use status <- decode.field("globalStatus", types.system_status_decoder())
    decode.success(status)
  }
  rockbox.query(
    client,
    "query GlobalStatus {
       globalStatus {
         resumeIndex resumeCrc32 resumeElapsed resumeOffset
         runtime topruntime dircacheSize
         lastScreen viewerIconCount lastVolumeChange
       }
     }",
    json.object([]),
    decoder,
  )
}

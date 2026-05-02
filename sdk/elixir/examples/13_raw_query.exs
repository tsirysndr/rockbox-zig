# 13 — Raw GraphQL escape hatch
#
# For operations not yet covered by a dedicated SDK function, drop down to
# `Rockbox.query/3` (snake_case variables are converted to camelCase).
#
#   mix run examples/13_raw_query.exs

Code.require_file("_helper.exs", __DIR__)

client = Examples.Helper.client()

{:ok, %{"rockboxVersion" => v}} = Rockbox.query(client, "query { rockboxVersion }")
IO.puts("rockboxd #{v}")

{:ok, %{"album" => album}} =
  Rockbox.query(
    client,
    """
    query Album($id: String!) { album(id: $id) { id title artist year } }
    """,
    id: "demo-id-or-use-a-real-one"
  )

IO.inspect(album, label: "album")

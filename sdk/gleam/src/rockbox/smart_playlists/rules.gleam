//// Composable builder for smart-playlist rule sets. Pipe-friendly.
////
//// ```gleam
//// import rockbox/smart_playlists
//// import rockbox/smart_playlists/rules
////
//// let r =
////   rules.all_of()
////   |> rules.where("play_count", rules.Gte, rules.int(10))
////   |> rules.where("last_played", rules.Within, rules.string("30d"))
////   |> rules.sort("play_count", rules.Desc)
////   |> rules.limit(50)
////
//// let input = smart_playlists.new("Most played", rules.to_string(r))
//// let assert Ok(_) = smart_playlists.create(client, input)
//// ```
////
//// `any_of()` swaps the top-level operator from `AND` to `OR`. Mix the two by
//// nesting builders with `where_group`:
////
//// ```gleam
//// let r =
////   rules.all_of()
////   |> rules.where("genre", rules.Eq, rules.string("Rock"))
////   |> rules.where_group(
////     rules.any_of()
////     |> rules.where("year", rules.Gte, rules.int(2000))
////     |> rules.where("year", rules.Lte, rules.int(2010)),
////   )
//// ```

import gleam/json.{type Json}
import gleam/list
import gleam/option.{type Option, None, Some}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A composable rule set. Build with `all_of` / `any_of` and chain
/// `where` / `sort` / `limit`. Hand to `to_string` to get the JSON payload
/// the GraphQL `createSmartPlaylist` mutation expects.
pub opaque type Rules {
  Rules(
    operator: GroupOp,
    rules: List(Node),
    sort: Option(Sort),
    limit: Option(Int),
  )
}

/// How sibling rules combine.
pub type GroupOp {
  And
  Or
}

/// Comparison operators understood by the server.
///
/// | Variant     | Meaning                                |
/// |-------------|----------------------------------------|
/// | `Eq`        | equals                                 |
/// | `Neq`       | not equals                             |
/// | `Gt`        | greater than                           |
/// | `Gte`       | greater than or equal                  |
/// | `Lt`        | less than                              |
/// | `Lte`       | less than or equal                     |
/// | `Contains`  | substring match                        |
/// | `Within`    | duration window (e.g. `"30d"`, `"7d"`) |
pub type Op {
  Eq
  Neq
  Gt
  Gte
  Lt
  Lte
  Contains
  Within
}

/// Sort direction.
pub type SortDir {
  Asc
  Desc
}

/// Wrapped value handed to `where`. Build with `int` / `string` / `bool` /
/// `float` / `null`, or `raw` if you have a `Json` already.
pub opaque type Value {
  Value(Json)
}

type Sort {
  Sort(field: String, dir: SortDir)
}

type Node {
  Leaf(field: String, op: Op, value: Json)
  Group(Rules)
}

// ---------------------------------------------------------------------------
// Value constructors
// ---------------------------------------------------------------------------

/// Wrap an integer (e.g. play counts, years, durations in ms).
pub fn int(value: Int) -> Value {
  Value(json.int(value))
}

/// Wrap a string (e.g. titles, artists, genres, duration windows).
pub fn string(value: String) -> Value {
  Value(json.string(value))
}

/// Wrap a boolean.
pub fn bool(value: Bool) -> Value {
  Value(json.bool(value))
}

/// Wrap a float.
pub fn float(value: Float) -> Value {
  Value(json.float(value))
}

/// JSON `null`.
pub fn null() -> Value {
  Value(json.null())
}

/// Escape hatch for arbitrary JSON values (arrays, nested objects, etc.).
pub fn raw(value: Json) -> Value {
  Value(value)
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

/// Start a builder where every rule must match (logical AND).
pub fn all_of() -> Rules {
  Rules(operator: And, rules: [], sort: None, limit: None)
}

/// Start a builder where any single rule must match (logical OR).
pub fn any_of() -> Rules {
  Rules(operator: Or, rules: [], sort: None, limit: None)
}

/// Append a rule to the current group.
pub fn where(
  builder: Rules,
  field: String,
  op: Op,
  value: Value,
) -> Rules {
  let Value(json_value) = value
  let leaf = Leaf(field: field, op: op, value: json_value)
  Rules(..builder, rules: list.append(builder.rules, [leaf]))
}

/// Nest another builder underneath this one. Useful for mixing AND / OR.
pub fn where_group(parent: Rules, child: Rules) -> Rules {
  Rules(..parent, rules: list.append(parent.rules, [Group(child)]))
}

/// Set the result ordering.
pub fn sort(builder: Rules, field: String, direction: SortDir) -> Rules {
  Rules(..builder, sort: Some(Sort(field: field, dir: direction)))
}

/// Cap the result count.
pub fn limit(builder: Rules, count: Int) -> Rules {
  Rules(..builder, limit: Some(count))
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Encode the builder as a `Json` value — useful if you're already working
/// with JSON values.
pub fn to_json(builder: Rules) -> Json {
  let base = [
    #("operator", json.string(group_op_to_string(builder.operator))),
    #("rules", json.array(builder.rules, node_to_json)),
  ]

  let with_sort = case builder.sort {
    Some(Sort(field, dir)) ->
      list.append(base, [
        #(
          "sort",
          json.object([
            #("field", json.string(field)),
            #("dir", json.string(sort_dir_to_string(dir))),
          ]),
        ),
      ])
    None -> base
  }

  let with_limit = case builder.limit {
    Some(n) -> list.append(with_sort, [#("limit", json.int(n))])
    None -> with_sort
  }

  json.object(with_limit)
}

/// Encode the builder as a JSON string ready for the smart-playlist mutation.
pub fn to_string(builder: Rules) -> String {
  json.to_string(to_json(builder))
}

// ---------------------------------------------------------------------------
// Internal encoding
// ---------------------------------------------------------------------------

fn node_to_json(node: Node) -> Json {
  case node {
    Leaf(field, op, value) ->
      json.object([
        #("field", json.string(field)),
        #("op", json.string(op_to_string(op))),
        #("value", value),
      ])
    Group(child) -> to_json(child)
  }
}

fn group_op_to_string(op: GroupOp) -> String {
  case op {
    And -> "AND"
    Or -> "OR"
  }
}

fn op_to_string(op: Op) -> String {
  case op {
    Eq -> "eq"
    Neq -> "neq"
    Gt -> "gt"
    Gte -> "gte"
    Lt -> "lt"
    Lte -> "lte"
    Contains -> "contains"
    Within -> "within"
  }
}

fn sort_dir_to_string(dir: SortDir) -> String {
  case dir {
    Asc -> "asc"
    Desc -> "desc"
  }
}

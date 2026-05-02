# frozen_string_literal: true

require_relative "../test_helper"

class LibraryApiTest < Minitest::Test
  def setup
    @http    = Rockbox::FakeHttp.new
    @library = Rockbox::Api::Library.new(@http)
  end

  def test_albums_maps_to_album_structs
    @http.will_return(albums: [
      { id: "a1", title: "Wish You Were Here" },
      { id: "a2", title: "The Wall" }
    ])
    albums = @library.albums
    assert_equal 2, albums.size
    assert_kind_of Rockbox::Album, albums.first
    assert_equal "Wish You Were Here", albums.first.title
  end

  def test_albums_with_nil_response
    @http.will_return(albums: nil)
    assert_equal [], @library.albums
  end

  def test_album_returns_nil_when_not_found
    @http.will_return(album: nil)
    assert_nil @library.album("missing")
  end

  def test_album_passes_id_variable
    @http.will_return(album: { id: "a1", title: "Wish You Were Here" })
    @library.album("a1")
    assert_equal({ id: "a1" }, @http.last_call.variables)
  end

  def test_like_album_passes_id
    @library.like_album("a1")
    assert_equal({ id: "a1" }, @http.last_call.variables)
  end

  def test_search_assembles_search_results
    @http.will_return(search: {
      artists:      [{ id: "ar1", name: "Pink Floyd" }],
      albums:       [{ id: "al1", title: "Animals" }],
      tracks:       [{ id: "t1",  title: "Pigs" }],
      liked_tracks: [],
      liked_albums: []
    })
    results = @library.search("pink floyd")
    assert_kind_of Rockbox::SearchResults, results
    assert_equal "Pink Floyd", results.artists.first.name
    assert_equal "Animals",    results.albums.first.title
    assert_equal "Pigs",       results.tracks.first.title
  end

  def test_search_when_field_is_nil
    @http.will_return(search: nil)
    results = @library.search("x")
    assert_equal [], results.tracks
    assert_equal [], results.albums
    assert_equal [], results.artists
  end

  def test_scan_emits_no_variables
    @library.scan
    assert_nil @http.last_call.variables
    assert_match(/scanLibrary/, @http.last_call.query)
  end
end

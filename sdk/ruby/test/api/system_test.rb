# frozen_string_literal: true

require_relative "../test_helper"

class SystemApiTest < Minitest::Test
  def setup
    @http   = Rockbox::FakeHttp.new
    @system = Rockbox::Api::System.new(@http)
  end

  def test_version_returns_string
    @http.will_return(rockbox_version: "Rockbox v3.15")
    assert_equal "Rockbox v3.15", @system.version
  end

  def test_status_builds_struct
    @http.will_return(global_status: {
      resume_index: 0, resume_crc32: 0, resume_elapsed: 0, resume_offset: 0,
      runtime: 100, topruntime: 200, dircache_size: 0,
      last_screen: 0, viewer_icon_count: 0, last_volume_change: 0
    })
    status = @system.status
    assert_kind_of Rockbox::SystemStatus, status
    assert_equal 100, status.runtime
    assert_equal 200, status.topruntime
  end
end

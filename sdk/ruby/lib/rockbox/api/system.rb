# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class System
      def initialize(http)
        @http = http
      end

      # @return [String]
      def version
        @http.execute("query Version { rockboxVersion }")[:rockbox_version]
      end

      # @return [Rockbox::SystemStatus]
      def status
        data = @http.execute(<<~GQL)
          query GlobalStatus {
            globalStatus {
              resumeIndex resumeCrc32 resumeElapsed resumeOffset
              runtime topruntime dircacheSize
              lastScreen viewerIconCount lastVolumeChange
            }
          }
        GQL
        SystemStatus.from_hash(data[:global_status])
      end
    end
  end
end

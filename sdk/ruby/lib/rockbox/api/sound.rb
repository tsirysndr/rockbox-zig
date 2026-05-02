# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Sound
      def initialize(http)
        @http = http
      end

      # @return [Rockbox::VolumeInfo]
      def volume
        data = @http.execute("query Volume { volume { volume min max } }")
        VolumeInfo.from_hash(data[:volume])
      end

      # Adjust volume by N steps (positive = louder, negative = quieter).
      # @return [Integer] resulting volume
      def adjust(steps)
        @http.execute(
          "mutation AdjustVolume($steps: Int!) { adjustVolume(steps: $steps) }",
          { steps: steps }
        )[:adjust_volume]
      end

      def up;   adjust(1);  end
      def down; adjust(-1); end
    end
  end
end

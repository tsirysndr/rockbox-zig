# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Browse
      def initialize(http)
        @http = http
      end

      # @param path [String, nil] absolute path; nil for the music root.
      # @return [Array<Rockbox::Entry>]
      def entries(path = nil)
        data = @http.execute(
          "query Browse($path: String) { treeGetEntries(path: $path) { name attr timeWrite customaction displayName } }",
          path ? { path: path } : nil
        )
        Array(data[:tree_get_entries]).map { |e| Entry.from_hash(e) }
      end

      def directories(path = nil)
        entries(path).select { |e| Rockbox.directory?(e) }
      end

      def files(path = nil)
        entries(path).reject { |e| Rockbox.directory?(e) }
      end
    end
  end
end

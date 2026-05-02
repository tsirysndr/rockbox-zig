# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Devices
      FIELDS = "id name host ip port service app isConnected baseUrl isCastDevice isSourceDevice isCurrentDevice"

      def initialize(http)
        @http = http
      end

      def list
        data = @http.execute("query Devices { devices { #{FIELDS} } }")
        Array(data[:devices]).map { |d| Device.from_hash(d) }
      end

      def get(id)
        data = @http.execute(
          "query Device($id: String!) { device(id: $id) { #{FIELDS} } }",
          { id: id }
        )
        Device.from_hash(data[:device])
      end

      def connect(id)
        @http.execute("mutation ConnectDevice($id: String!) { connect(id: $id) }", { id: id })
        nil
      end

      def disconnect(id)
        @http.execute("mutation DisconnectDevice($id: String!) { disconnect(id: $id) }", { id: id })
        nil
      end
    end
  end
end

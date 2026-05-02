# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Bluetooth
      FIELDS = <<~GQL
        fragment BluetoothDeviceFields on BluetoothDevice {
          address name paired trusted connected rssi
        }
      GQL

      def initialize(http)
        @http = http
      end

      # List paired/known Bluetooth devices (Linux only).
      def devices
        data = @http.execute("#{FIELDS}\nquery BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }")
        Array(data[:bluetooth_devices]).map { |d| BluetoothDevice.from_hash(d) }
      end

      # Scan for nearby devices (Linux only).
      def scan(timeout: nil)
        data = @http.execute(<<~GQL, timeout ? { timeout_secs: timeout } : nil)
          #{FIELDS}
          mutation BluetoothScan($timeoutSecs: Int) {
            bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields }
          }
        GQL
        Array(data[:bluetooth_scan]).map { |d| BluetoothDevice.from_hash(d) }
      end

      def connect(address)
        @http.execute(
          "mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }",
          { address: address }
        )
        nil
      end

      def disconnect(address)
        @http.execute(
          "mutation BluetoothDisconnect($address: String!) { bluetoothDisconnect(address: $address) }",
          { address: address }
        )
        nil
      end
    end
  end
end

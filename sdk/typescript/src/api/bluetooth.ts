import type { HttpTransport } from '../transport.js';
import type { BluetoothDevice } from '../types.js';

const BLUETOOTH_DEVICE_FIELDS = /* GraphQL */ `
  fragment BluetoothDeviceFields on BluetoothDevice {
    address name paired trusted connected rssi
  }
`;

export class BluetoothApi {
  constructor(private readonly http: HttpTransport) {}

  /** List paired/known Bluetooth devices (Linux only) */
  async devices(): Promise<BluetoothDevice[]> {
    const data = await this.http.execute<{ bluetoothDevices: BluetoothDevice[] }>(/* GraphQL */ `
      ${BLUETOOTH_DEVICE_FIELDS}
      query BluetoothDevices { bluetoothDevices { ...BluetoothDeviceFields } }
    `);
    return data.bluetoothDevices;
  }

  /** Scan for nearby Bluetooth devices (Linux only) */
  async scan(timeoutSecs?: number): Promise<BluetoothDevice[]> {
    const data = await this.http.execute<{ bluetoothScan: BluetoothDevice[] }>(/* GraphQL */ `
      ${BLUETOOTH_DEVICE_FIELDS}
      mutation BluetoothScan($timeoutSecs: Int) {
        bluetoothScan(timeoutSecs: $timeoutSecs) { ...BluetoothDeviceFields }
      }
    `, { timeoutSecs });
    return data.bluetoothScan;
  }

  /** Connect to a Bluetooth device by address (Linux only) */
  async connect(address: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation BluetoothConnect($address: String!) { bluetoothConnect(address: $address) }
    `, { address });
  }

  /** Disconnect a Bluetooth device by address (Linux only) */
  async disconnect(address: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation BluetoothDisconnect($address: String!) { bluetoothDisconnect(address: $address) }
    `, { address });
  }
}

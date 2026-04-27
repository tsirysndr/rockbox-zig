import type { HttpTransport } from '../transport.js';
import type { Device } from '../types.js';

export class DevicesApi {
  constructor(private readonly http: HttpTransport) {}

  async list(): Promise<Device[]> {
    const data = await this.http.execute<{ devices: Device[] }>(/* GraphQL */ `
      query Devices {
        devices {
          id name host ip port service app isConnected
          baseUrl isCastDevice isSourceDevice isCurrentDevice
        }
      }
    `);
    return data.devices;
  }

  async get(id: string): Promise<Device | null> {
    const data = await this.http.execute<{ device: Device | null }>(/* GraphQL */ `
      query Device($id: String!) {
        device(id: $id) {
          id name host ip port service app isConnected
          baseUrl isCastDevice isSourceDevice isCurrentDevice
        }
      }
    `, { id });
    return data.device;
  }

  async connect(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation ConnectDevice($id: String!) { connect(id: $id) }
    `, { id });
  }

  async disconnect(id: string): Promise<void> {
    await this.http.execute(/* GraphQL */ `
      mutation DisconnectDevice($id: String!) { disconnect(id: $id) }
    `, { id });
  }
}

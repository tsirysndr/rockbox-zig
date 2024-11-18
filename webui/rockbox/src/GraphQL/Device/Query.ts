import { gql } from "@apollo/client";

export const GET_DEVICES = gql`
  query GetDevices {
    devices {
      id
      name
      app
      ip
      host
      port
      isCastDevice
      service
      isConnected
    }
  }
`;

export const GET_DEVICE = gql`
  query GetDevice($id: String!) {
    device(id: $id) {
      id
      name
      app
      ip
      host
      port
      isCastDevice
      service
      isConnected
    }
  }
`;

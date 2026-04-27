import { gql } from "graphql-tag";

export const CONNECT_TO_DEVICE = gql`
  mutation ConnectToDevice($id: String!) {
    connect(id: $id)
  }
`;

export const DISCONNECT_FROM_DEVICE = gql`
  mutation DisconnectFromDevice($id: String!) {
    disconnect(id: $id)
  }
`;

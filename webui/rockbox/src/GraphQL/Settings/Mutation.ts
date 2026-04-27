import { gql } from "graphql-tag";

export const SAVE_SETTINGS = gql`
  mutation SaveSettings($settings: NewGlobalSettings!) {
    saveSettings(settings: $settings) 
  }
`;
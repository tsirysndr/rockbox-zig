import { gql } from "@apollo/client";

export const SAVE_SETTINGS = gql`
  mutation SaveSettings($settings: NewGlobalSettings!) {
    saveSettings(settings: $settings) 
  }
`;
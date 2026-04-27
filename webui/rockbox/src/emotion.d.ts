import "@emotion/react";

declare module "@emotion/react" {
  export interface Theme {
    colors: {
      primary: string;
      text: string;
      background: string;
      surface: string;
      icon: string;
      searchBackground: string;
      searchBackgroundAlt: string;
      secondaryBackground: string;
      secondaryText: string;
      backButton: string;
      currentTrackBorder: string;
      popoverBackground: string;
      separator: string;
      hover: string;
      cover: string;
      tooltip: string;
    };
  }
}

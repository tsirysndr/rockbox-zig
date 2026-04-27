import { createLightTheme, createDarkTheme } from "baseui/themes";

const PRIMARY = "#6F00FF";
const PRIMARY_HOVER = "#6F00FFbe";
const PRIMARY_SECONDARY_FILL = "rgba(111, 0, 255, 0.08)";
const PRIMARY_SECONDARY_HOVER = "rgba(111, 0, 255, 0.14)";

export const BaseUIDarkTheme = createDarkTheme(
  {
    primaryFontFamily: "RockfordSansRegular",
  },
  {
    colors: {
      buttonPrimaryFill: PRIMARY,
      buttonPrimaryText: "#fff",
      buttonPrimaryHover: PRIMARY_HOVER,
      buttonSecondaryFill: PRIMARY_SECONDARY_FILL,
      buttonSecondaryText: PRIMARY,
      buttonSecondaryHover: PRIMARY_SECONDARY_HOVER,
      buttonTertiaryText: "#fff",
      buttonTertiaryFill: "#1a1a1a",
      buttonTertiaryHover: "#2a2a2a",
    },
  }
);

export const BaseUILightTheme = createLightTheme(
  {
    primaryFontFamily: "RockfordSansRegular",
  },
  {
    colors: {
      buttonPrimaryFill: PRIMARY,
      buttonPrimaryHover: PRIMARY_HOVER,
      buttonSecondaryFill: PRIMARY_SECONDARY_FILL,
      buttonSecondaryText: PRIMARY,
      buttonSecondaryHover: PRIMARY_SECONDARY_HOVER,
      buttonTertiaryText: "#fff",
      buttonTertiaryFill: "#000",
      buttonTertiaryHover: "#00000097",
    },
  }
);

export const LightTheme = {
  colors: {
    primary: PRIMARY,
    text: "#000",
    background: "#fff",
    surface: "#f6f9fc",
    icon: "#000",
    searchBackground: "rgba(247, 247, 247, 0.2)",
    searchBackgroundAlt: "rgba(255, 255, 255, 0.2)",
    secondaryBackground: "#f0e8ff",
    secondaryText: "rgba(0, 0, 0, 0.542)",
    backButton: "#f0f0f1",
    currentTrackBorder: "rgba(177, 178, 181, 0.25)",
    popoverBackground: "#fff",
    separator: "#e0e0e0cc",
    hover: "#b1b2b51a",
    cover: "#f3f3f3b9",
    tooltip: `${PRIMARY}1a`,
  },
};

export const DarkTheme = {
  colors: {
    primary: PRIMARY,
    text: "#e8e8e8",
    background: "#0a0a0a",
    surface: "#111116",
    icon: "#9090a0",
    searchBackground: "#141418",
    searchBackgroundAlt: "#141418",
    secondaryBackground: "#1a1228",
    secondaryText: "#7878a0",
    backButton: "#1e1e24",
    currentTrackBorder: "rgba(255, 255, 255, 0.08)",
    popoverBackground: "#18181e",
    separator: "rgba(255, 255, 255, 0.08)",
    hover: "#1e1e24",
    cover: "#28282e",
    tooltip: `${PRIMARY}30`,
  },
};

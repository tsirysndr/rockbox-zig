/**
 * Rockbox theme — single dark palette mirroring gpui/src/ui/theme.rs,
 * tuned with a Spotify / Tidal aesthetic and the Rockbox accent purple (#6F00FF).
 */

export const Colors = {
  appBg: "#000000",
  bgElevated: "#15171F",
  bgCard: "#1A1D26",
  bgHover: "#22252F",
  bgDock: "#0F1014",

  accent: "#6F00FF",
  accentSoft: "#1A0E3D",
  accentHover: "rgba(111,0,255,0.9)",

  textPrimary: "#FFFFFF",
  textSecondary: "#9898A8",
  textMuted: "#9090A3",

  border: "rgba(255,255,255,0.16)",
  divider: "rgba(255,255,255,0.10)",

  sliderTrack: "rgba(255,255,255,0.10)",
  sliderFill: "#6F00FF",

  iconIdle: "#9090A3",
  iconActive: "#6F00FF",

  // Compatibility shape for components that still expect Colors[scheme].x
  light: {
    text: "#FFFFFF",
    background: "#0F1117",
    tint: "#6F00FF",
    icon: "#9090A3",
    tabIconDefault: "#9090A3",
    tabIconSelected: "#FFFFFF",
  },
  dark: {
    text: "#FFFFFF",
    background: "#0F1117",
    tint: "#6F00FF",
    icon: "#9090A3",
    tabIconDefault: "#9090A3",
    tabIconSelected: "#FFFFFF",
  },
};

export const Fonts = {
  sans: "SpaceGrotesk",
  mono: "JetBrainsMono",
};

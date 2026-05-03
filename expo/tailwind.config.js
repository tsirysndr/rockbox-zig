/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,jsx,ts,tsx}",
    "./components/**/*.{js,jsx,ts,tsx}",
  ],
  presets: [require("nativewind/preset")],
  theme: {
    extend: {
      colors: {
        // Rockbox dark palette — keep in sync with constants/theme.ts
        bg: {
          DEFAULT: "#000000",
          elevated: "#15171F",
          card: "#1A1D26",
          hover: "#22252F",
          dock: "#0F1014",
        },
        accent: {
          DEFAULT: "#6F00FF",
          soft: "#1A0E3D",
          hover: "rgba(111,0,255,0.9)",
        },
        text: {
          primary: "#FFFFFF",
          secondary: "#9898A8",
          muted: "#9090A3",
        },
        divider: "rgba(255,255,255,0.10)",
        border: "rgba(255,255,255,0.16)",
        slider: {
          track: "rgba(255,255,255,0.10)",
          fill: "#6F00FF",
        },
        danger: "#FF6B6B",
      },
      fontFamily: {
        sans: ["SpaceGrotesk"],
        mono: ["JetBrainsMono"],
      },
    },
  },
  plugins: [],
};

import { createContext, useState, FC } from "react";
import { ThemeProvider as EmotionThemeProvider, Global, css } from "@emotion/react";
import {
  BaseUIDarkTheme,
  BaseUILightTheme,
  DarkTheme,
  LightTheme,
} from "../Theme";
import { BaseProvider } from "baseui";
import { ThemeProvider as MaterialThemeProvider } from "@mui/material/styles";
import { createTheme } from "@mui/material/styles";

export type Theme = "light" | "dark";

export type ThemeContextType = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

export const ThemeContext = createContext<ThemeContextType>({
  theme: "dark",
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  setTheme: (_theme: Theme) => {},
});

const muitheme = createTheme({
  cssVariables: true,
  palette: {
    primary: {
      main: "#6F00FF",
    },
  },
});

export type ThemeProviderProps = {
  children: React.ReactNode;
};

const ThemeProvider: FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setTheme] = useState<Theme>("dark");
  const emotionTheme = theme === "dark" ? DarkTheme : LightTheme;
  return (
    <MaterialThemeProvider theme={muitheme}>
      <ThemeContext.Provider value={{ theme, setTheme }}>
        <EmotionThemeProvider theme={emotionTheme}>
          <Global
            styles={css`
              body {
                background-color: ${emotionTheme.colors.background};
                color: ${emotionTheme.colors.text};
              }
              :root {
                --row-hover-bg: ${emotionTheme.colors.hover};
                --text-color: ${emotionTheme.colors.text};
                --secondary-text: ${emotionTheme.colors.secondaryText};
                --separator-color: ${emotionTheme.colors.separator};
              }
              tbody > tr:hover {
                background-color: ${emotionTheme.colors.hover} !important;
              }
            `}
          />
          <BaseProvider
            theme={theme === "dark" ? BaseUIDarkTheme : BaseUILightTheme}
          >
            {children}
          </BaseProvider>
        </EmotionThemeProvider>
      </ThemeContext.Provider>
    </MaterialThemeProvider>
  );
};

export default ThemeProvider;

import { createContext, useState, FC } from "react";
import { ThemeProvider as EmotionThemeProvider } from "@emotion/react";
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
  theme: "light",
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  setTheme: (_theme: Theme) => {},
});

const muitheme = createTheme({
  cssVariables: true,
  palette: {
    primary: {
      main: "#fe099c",
    },
  },
});

export type ThemeProviderProps = {
  children: React.ReactNode;
};

const ThemeProvider: FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setTheme] = useState<Theme>("light");
  return (
    <MaterialThemeProvider theme={muitheme}>
      <ThemeContext.Provider value={{ theme, setTheme }}>
        <EmotionThemeProvider theme={theme === "dark" ? DarkTheme : LightTheme}>
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

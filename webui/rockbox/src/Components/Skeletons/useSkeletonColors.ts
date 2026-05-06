import { useContext } from "react";
import { ThemeContext } from "../../Providers/ThemeProvider";

const useSkeletonColors = () => {
  const { theme } = useContext(ThemeContext);
  const isDark = theme === "dark";
  return {
    backgroundColor: isDark ? "#1e1e24" : "#e0e0e0",
    foregroundColor: isDark ? "#2e2e3a" : "#ebebeb",
  };
};

export default useSkeletonColors;

import { useTheme } from "@emotion/react";

const useSkeletonColors = () => {
  const theme = useTheme();
  const isDark = theme.colors.text === "#e8e8e8";
  return {
    backgroundColor: isDark ? "#1e1e24" : "#e0e0e0",
    foregroundColor: isDark ? "#2e2e3a" : "#ebebeb",
  };
};

export default useSkeletonColors;

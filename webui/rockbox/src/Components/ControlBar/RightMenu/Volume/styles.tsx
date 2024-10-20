import styled from "@emotion/styled";

export const Container = styled.div`
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
`;

export default {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  slider: (t: any) => ({
    color: "rgba(0, 0, 0, 0.682)",
    "& .MuiSlider-track": {
      border: "none",
    },
    "& .MuiSlider-thumb": {
      width: 18,
      height: 18,
      backgroundColor: "#fff",
      "&::before": {
        boxShadow: "0 4px 8px rgba(0,0,0,0.18)",
      },
      "&:hover, &.Mui-focusVisible, &.Mui-active": {
        boxShadow: "none",
      },
    },
    ...t.applyStyles("dark", {
      color: "#fff",
    }),
  }),
};

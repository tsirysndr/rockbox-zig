import styled from "@emotion/styled";

export const Container = styled.div`
  height: 250px;
  margin: 0 auto;
  margin-top: 50px;
  margin-bottom: 120px;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  width: 73%;
  font-size: 13px;
`;

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  margin: auto;
  margin-bottom: 40px;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const Wrapper = styled.div`
  width: 60vw;
  margin: 0 auto;
  margin-bottom: 100px;
  margin-top: 30px;
`;

export const SettingsTitle = styled.div`
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 5px;
`;

export const Section = styled.div`
  margin-bottom: 50px;
  font-size: 15px;
`;

export const Item = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  height: 50px;
`;

const iOSBoxShadow =
  "0 3px 1px rgba(0,0,0,0.1),0 4px 8px rgba(0,0,0,0.13),0 0 0 1px rgba(0,0,0,0.02)";

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
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  sliderIOS: (theme: any) => ({
    color: "rgb(254, 9, 156)",

    "& .MuiSlider-thumb": {
      height: 20,
      width: 20,
      backgroundColor: "#fff",
      boxShadow: "0 0 2px 0px rgba(0, 0, 0, 0.1)",
      "&:focus, &:hover, &.Mui-active": {
        boxShadow: "0px 0px 3px 1px rgba(0, 0, 0, 0.1)",
        // Reset on touch devices, it doesn't add specificity
        "@media (hover: none)": {
          boxShadow: iOSBoxShadow,
        },
      },
      "&:before": {
        boxShadow:
          "0px 0px 1px 0px rgba(0,0,0,0.2), 0px 0px 0px 0px rgba(0,0,0,0.14), 0px 0px 1px 0px rgba(0,0,0,0.12)",
      },
    },
    "& .MuiSlider-track": {
      border: "none",
      height: 5,
    },
    "& .MuiSlider-rail": {
      opacity: 0.5,
      boxShadow: "inset 0px 0px 4px -2px #000",
      backgroundColor: "#d0d0d0",
    },
    ...theme.applyStyles("dark", {
      color: "rgb(254, 9, 156)",
    }),
  }),
};

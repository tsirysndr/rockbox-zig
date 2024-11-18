import { css } from "@emotion/react";
import styled from "@emotion/styled";

export const Container = styled.div`
  max-height: calc(100vh - 153px); /* - 90px */
  padding-top: 15px;
  padding-bottom: 15px;
  overflow-y: auto;
  width: 370px;
  min-height: 200px;
`;

export const List = styled.div`
  max-height: calc(100vh - 273px); /* - 210px */
  padding-left: 15px;
  padding-right: 15px;
  overflow-y: auto;
  min-height: 200px;
`;

export const Icon = styled.div`
  height: 40px;
  width: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: ${(props) => props.theme.colors.cover};
  ${(props) =>
    props.color &&
    css`
      background-color: ${props.color};
    `}
`;

export const Title = styled.div`
  margin: 10px;
  margin-left: 25px;
  margin-right: 25px;
  font-family: "RockfordSansBold";
`;

export const CurrentDeviceWrapper = styled.div`
  height: 60px;
  display: flex;
  margin-left: 25px;
  margin-right: 25px;
  align-items: center;
`;

export const CurrentDevice = styled.div`
  font-size: 18px;
`;

export const CurrentDeviceName = styled.div`
  color: #fe099c;
  font-size: 14px;
`;

export const IconWrapper = styled.div`
  margin-top: 3px;
  margin-right: 16px;
`;

export const Disconnect = styled.button`
  background-color: #000;
  border: none;
  color: #fff;
  height: 21px;
  border-radius: 12px;
  font-family: "RockfordSansRegular";
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  padding-bottom: 4px;
  cursor: pointer;
`;

export const Placeholder = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 300px;
  text-align: center;
  padding-left: 20px;
  padding-right: 20px;
  font-size: 14px;
`;

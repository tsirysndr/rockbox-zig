import logger from "npm:node-color-log@11.0.2";

export const log = (stack?: string) =>
  logger.bgColor("red").color("black").log(stack);

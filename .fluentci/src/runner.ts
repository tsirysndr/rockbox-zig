import pipeline from "./pipeline.ts";
import { parse, camelCase, snakeCase } from "../deps.ts";

const args = parse(Deno.args.map((x) => x.split(" ")).flat());

if (!Array.isArray(Deno.args)) {
  for (const param of Object.keys(args)
    .filter((x) => x !== "_")
    .map((x) => snakeCase(x).toUpperCase())) {
    Deno.env.set(param, args[camelCase(param)]);
  }
}

await pipeline(
  ".",
  Array.isArray(Deno.args) ? Deno.args : (args._ as string[])
);

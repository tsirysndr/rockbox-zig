import * as console from "ext:deno_console/01_console.js";

Object.defineProperty(globalThis, "console", {
  value: new console.Console((msg, level) =>
    globalThis.Deno.core.print(msg, level > 1)
  ),
  enumerable: false,
  configurable: true,
  writable: true,
});

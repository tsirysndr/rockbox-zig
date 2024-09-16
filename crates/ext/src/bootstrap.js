import "ext:deno_console/01_console.js";
import "ext:rockbox/src/sound/sound.js";
import "ext:rockbox/src/playlist/playlist.js";
import "ext:rockbox/src/browse/browse.js";
import "ext:rockbox/src/playback/playback.js";
import "ext:rockbox/src/settings/settings.js";
import "ext:rockbox/src/system/system.js";

import * as headers from "ext:deno_fetch/20_headers.js";
import * as formData from "ext:deno_fetch/21_formdata.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";
import * as eventSource from "ext:deno_fetch/27_eventsource.js";
import * as console from "ext:deno_console/01_console.js";
import * as url from "ext:deno_url/00_url.js";
import * as urlPattern from "ext:deno_url/01_urlpattern.js";
import * as webidl from "ext:deno_webidl/00_webidl.js";
import * as net from "ext:deno_net/01_net.js";
import * as tls from "ext:deno_net/02_tls.js";
import * as infra from "ext:deno_web/00_infra.js";
import * as DOMException from "ext:deno_web/01_dom_exception.js";
import * as mimesniff from "ext:deno_web/01_mimesniff.js";
import * as event from "ext:deno_web/02_event.js";
import * as structuredClone from "ext:deno_web/02_structured_clone.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as abortSignal from "ext:deno_web/03_abort_signal.js";
import * as globalInterfaces from "ext:deno_web/04_global_interfaces.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as file from "ext:deno_web/09_file.js";
import * as fileReader from "ext:deno_web/10_filereader.js";
import * as location from "ext:deno_web/12_location.js";
import * as messagePort from "ext:deno_web/13_message_port.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as performance from "ext:deno_web/15_performance.js";
import * as imageData from "ext:deno_web/16_image_data.js";

Object.defineProperty(globalThis, "AbortController", {
  value: abortSignal.AbortController,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, webidl.brand, {
  value: webidl.brand,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "URL", {
  value: url.URL,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "URLPattern", {
  value: url.URLPattern,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "URLSearchParams", {
  value: url.URLSearchParams,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "console", {
  value: new console.Console((msg, level) =>
    globalThis.Deno.core.print(msg, level > 1)
  ),
  enumerable: false,
  configurable: true,
  writable: true,
});

// Set up the callback for Wasm streaming ops
Deno.core.setWasmStreamingCallback(fetch.handleWasmStreaming);

Object.defineProperty(globalThis, "fetch", {
  value: fetch.fetch,
  enumerable: true,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "Request", {
  value: request.Request,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "Response", {
  value: response.Response,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "Headers", {
  value: headers.Headers,
  enumerable: false,
  configurable: true,
  writable: true,
});

Object.defineProperty(globalThis, "FormData", {
  value: formData.FormData,
  enumerable: false,
  configurable: true,
  writable: true,
});

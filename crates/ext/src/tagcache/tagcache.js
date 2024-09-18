const { ops } = Deno.core;

const tagcache = {};

globalThis.rb = { ...globalThis.rb, tagcache };

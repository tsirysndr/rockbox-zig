const { ops } = Deno.core;

const system = {
  getGlobalStatus: () => ops.op_get_global_status(),
  getRockboxVersion: () => ops.op_get_rockbox_version(),
};

globalThis.rb = { ...globalThis.rb, system };

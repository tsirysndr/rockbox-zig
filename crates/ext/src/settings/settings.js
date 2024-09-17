const { ops } = Deno.core;

const settings = {
  getGlobalSettings: () => {
    return ops.op_get_global_settings();
  },
};

globalThis.rb = { ...globalThis.rb, settings };

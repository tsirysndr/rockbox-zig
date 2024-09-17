const { ops } = Deno.core;

const browse = {
  rockboxBrowse: () => {
    return ops.op_rockbox_browse();
  },
};

globalThis.rb = { browse };

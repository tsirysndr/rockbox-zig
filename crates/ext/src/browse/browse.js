const { core } = Deno;
const { ops } = core;

const browse = {
  rockboxBrowse: () => {
    return ops.op_rockbox_browse();
  },
};

globalThis.rb = { browse };

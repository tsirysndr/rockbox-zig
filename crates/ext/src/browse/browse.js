const { ops } = Deno.core;

const browse = {
  rockboxBrowse: () => {
    return ops.op_rockbox_browse();
  },
  tree: {
    getEntries: (path) => {
      return ops.op_tree_get_entries(path);
    },
  },
};

globalThis.rb = { browse };

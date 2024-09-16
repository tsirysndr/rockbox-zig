const settings = {
  getGlobalSettings: () => {
    return ops.op_rockbox_browse();
  },
};

globalThis.rb = { ...globalThis.rb, settings };

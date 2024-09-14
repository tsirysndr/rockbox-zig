import Client from "../../deps.ts";
import { connect } from "../connect.ts";
import { withDevbox } from "./steps.ts";

connect(async (client: Client) => {
  const ctr = withDevbox(
    client
      .pipeline("nix-installer")
      .container()
      .from("alpine")
      .withExec(["apk", "add", "curl", "bash"])
  )
    .withExec(["devbox", "global", "add", "gh"])
    .withExec(["sh", "-c", 'eval "$(devbox global shellenv)" && gh version']);

  const result = await ctr.stdout();

  console.log(result);
});

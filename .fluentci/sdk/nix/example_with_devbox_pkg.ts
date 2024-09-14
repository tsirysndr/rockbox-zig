import Client from "../../deps.ts";
import { connect } from "../connect.ts";
import { withDevboxExec, withPackageFromDevbox } from "./steps.ts";

connect(async (client: Client) => {
  const ctr = withDevboxExec(
    withPackageFromDevbox(
      client
        .pipeline("nix-installer")
        .container()
        .from("alpine")
        .withExec(["apk", "add", "curl", "bash"]),
      ["gh"]
    ),
    ["gh version"]
  );

  const result = await ctr.stdout();

  console.log(result);
});

import Client from "../../deps.ts";
import { connect } from "../connect.ts";
import { withNix } from "./steps.ts";

connect(async (client: Client) => {
  const ctr = withNix(
    client
      .pipeline("nix-installer")
      .container()
      .from("alpine")
      .withExec(["apk", "add", "curl"])
  );

  const result = await ctr.stdout();

  console.log(result);
});

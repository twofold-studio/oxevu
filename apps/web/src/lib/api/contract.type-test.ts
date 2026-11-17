// Compiled by svelte-check; never executed. An unused @ts-expect-error fails checking.
import * as api from "./generated/sdk.gen";

export function rejectsIncompatibleApiUsage() {
  // @ts-expect-error password is required by the Rust Credentials request.
  void api.login({ body: { username: "owner" } });
  // @ts-expect-error a route parameter is required.
  void api.scan({});
  // @ts-expect-error library kind is a closed Rust enum.
  void api.create({ body: { name: "Movies", kind: "music", paths: ["/media"] } });
  // @ts-expect-error backend path parameters are generated, not freely named.
  void api.downloadGet({ path: { download_id: "example" } });
}

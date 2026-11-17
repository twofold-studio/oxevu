import { spawnSync } from "node:child_process";
import { writeFileSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const artifacts = `${root}/apps/web/src/lib/api`;
function snapshot() {
  const files = ["openapi.json", ...readdirSync(`${artifacts}/generated`, { recursive: true })
    .filter((file) => file.endsWith(".ts")).map((file) => `generated/${file}`)];
  return JSON.stringify(files.sort().map((file) => [file, readFileSync(`${artifacts}/${file}`, "utf8")]));
}
const before = process.argv.includes("--check") ? snapshot() : null;
const result = spawnSync("cargo", ["run", "--locked", "--quiet", "-p", "oxevu-api", "--bin", "oxevu-export-api"], {
  cwd: root,
  encoding: "utf8",
  stdio: ["ignore", "pipe", "inherit"],
});
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
JSON.parse(result.stdout);
writeFileSync(new URL("../apps/web/src/lib/api/openapi.json", import.meta.url), result.stdout);
const generated = spawnSync("pnpm", ["gen:api"], { cwd: `${root}/apps/web`, stdio: "inherit" });
if (generated.error) throw generated.error;
if (generated.status !== 0) process.exit(generated.status ?? 1);
if (before !== null && before !== snapshot()) {
  console.error("API artifacts were stale. Review and commit the regenerated contract and SDK.");
  process.exit(1);
}

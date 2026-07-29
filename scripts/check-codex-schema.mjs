import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";

const schemaDirectory = process.argv[2];
if (!schemaDirectory) throw new Error("Usage: node scripts/check-codex-schema.mjs <generated-schema-directory>");

async function collect(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const contents = [];
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) contents.push(...(await collect(path)));
    else if (entry.name.endsWith(".json")) contents.push(await readFile(path, "utf8"));
  }
  return contents;
}

const contract = JSON.parse(await readFile(new URL("../src-tauri/codex-protocol.json", import.meta.url), "utf8"));
const generatedSchema = (await collect(schemaDirectory)).join("\n");
const missing = [...contract.methods, ...contract.responseFields].filter(
  (token) => !generatedSchema.includes(`\"${token}\"`),
);
if (missing.length > 0)
  throw new Error(`Bundled Codex App Server schema is incompatible; missing: ${missing.join(", ")}`);
console.log(
  `Codex App Server schema covers ${contract.methods.length} methods and ${contract.responseFields.length} fields.`,
);

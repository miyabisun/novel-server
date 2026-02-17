import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { createPrompt } from "./lib/prompt.js";

const ENV_PATH = ".env";
const EXAMPLE_PATH = ".env.example";

const { prompt, close } = createPrompt();

async function main() {
  if (existsSync(ENV_PATH)) {
    const overwrite = await prompt(".env already exists. Overwrite? [y/N]: ");
    if (overwrite.toLowerCase() !== "y") {
      console.log("Aborted.");
      return;
    }
  }

  if (!existsSync(EXAMPLE_PATH)) {
    console.error(".env.example not found.");
    process.exitCode = 1;
    return;
  }

  let env = readFileSync(EXAMPLE_PATH, "utf-8");

  writeFileSync(ENV_PATH, env);
  console.log(".env created successfully.");
}

await main().finally(close);

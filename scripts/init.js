import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { randomBytes } from "node:crypto";
import { createPrompt } from "./lib/prompt.js";

const ENV_PATH = ".env";
const EXAMPLE_PATH = ".env.example";

function generateSecret(length = 48) {
  return randomBytes(length).toString("base64");
}

function generatePassword(length = 16) {
  return randomBytes(length).toString("base64url").slice(0, length);
}

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

  const inputPassword = await prompt(
    "AUTH_PASSWORD (leave empty to auto-generate): "
  );
  if (inputPassword.includes('"') || inputPassword.includes("\n")) {
    console.error('AUTH_PASSWORD must not contain " or newlines.');
    process.exitCode = 1;
    return;
  }
  const password = inputPassword || generatePassword();
  if (!inputPassword) console.log("Generated AUTH_PASSWORD (see .env).");
  env = env.replace(/^AUTH_PASSWORD=.*$/m, `AUTH_PASSWORD=${password}`);

  const jwtSecret = generateSecret();
  console.log("Generated JWT_SECRET.");
  env = env.replace(/^JWT_SECRET=.*$/m, `JWT_SECRET=${jwtSecret}`);

  writeFileSync(ENV_PATH, env);
  console.log(".env created successfully.");
}

await main().finally(close);

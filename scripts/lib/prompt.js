import { createInterface } from "node:readline";

export function createPrompt() {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  const lines = [];
  let lineResolve = null;

  rl.on("line", (line) => {
    if (lineResolve) {
      const resolve = lineResolve;
      lineResolve = null;
      resolve(line);
    } else {
      lines.push(line);
    }
  });

  rl.on("close", () => {
    if (lineResolve) {
      const resolve = lineResolve;
      lineResolve = null;
      resolve("");
    }
  });

  function prompt(question) {
    process.stdout.write(question);
    if (lines.length > 0) {
      return Promise.resolve(lines.shift());
    }
    return new Promise((resolve) => {
      lineResolve = resolve;
    });
  }

  function close() {
    rl.close();
  }

  return { prompt, close };
}

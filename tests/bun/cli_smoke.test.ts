import { describe, expect, it } from "bun:test";

const decoder = new TextDecoder();

function runVoltts(args: string[]) {
  return Bun.spawnSync({
    cmd: ["cargo", "run", "--", ...args],
    stdout: "pipe",
    stderr: "pipe",
  });
}

describe("voltts CLI (bun smoke)", () => {
  it("prints help with bun test runner", () => {
    const result = runVoltts(["--help"]);

    const stdout = decoder.decode(result.stdout);
    const stderr = decoder.decode(result.stderr);

    if (result.exitCode !== 0) {
      throw new Error(`voltts --help failed: code=${result.exitCode}\nSTDOUT:\n${stdout}\nSTDERR:\n${stderr}`);
    }

    expect(stdout).toContain("voltts");
    expect(stdout).toContain("USAGE");
  });
});

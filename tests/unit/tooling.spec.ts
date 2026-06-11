import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";

function scriptEnv(extra: Record<string, string>) {
  const env = { ...process.env };
  for (const key of Object.keys(env)) {
    if (key.toLowerCase() === "npm_config_user_agent") {
      delete env[key];
    }
  }
  return { ...env, ...extra };
}

function readWorkspaceFile(path: string) {
  return readFileSync(resolve(path), "utf-8");
}

describe("Ameya tooling", () => {
  it("root package.json uses Yarn 4 and project verification scripts", () => {
    const pkg = JSON.parse(readWorkspaceFile("package.json"));

    expect(pkg.workspaces).toBeUndefined();
    expect(pkg.packageManager).toBe("yarn@4.14.1");
    expect(pkg.scripts).toMatchObject({
      "check:package-manager": "node scripts/check-package-manager.mjs",
      dev: "vite",
      build: "vue-tsc --noEmit && vite build",
      test: "vitest run",
      "docs:dev": "vitepress dev docs",
      "docs:build": "vitepress build docs",
      "docs:preview": "vitepress preview docs",
      tauri: "tauri",
      "tauri:dev": "node scripts/tauri-dev.mjs",
      "tauri:build": "tauri build",
      verify: "yarn test && yarn build && cargo test --manifest-path src-tauri/Cargo.toml",
    });
  });

  it("package-manager check accepts Yarn 4 and rejects other entrypoints", () => {
    const ok = spawnSync("node", ["scripts/check-package-manager.mjs"], {
      cwd: resolve("."),
      env: scriptEnv({ npm_config_user_agent: "yarn/4.14.1 npm/? node/?" }),
      encoding: "utf-8",
    });
    expect(ok.status).toBe(0);

    const bad = spawnSync("node", ["scripts/check-package-manager.mjs"], {
      cwd: resolve("."),
      env: scriptEnv({ npm_config_user_agent: "npm/11.0.0 node/?" }),
      encoding: "utf-8",
    });
    expect(bad.status).toBe(1);
    expect(bad.stderr).toContain("Ameya requires Yarn 4 through Corepack.");
  });

  it("Tauri dev script dry-run outputs dynamic Ameya port config", () => {
    const run = spawnSync("node", ["scripts/tauri-dev.mjs", "--verbose"], {
      cwd: resolve("."),
      env: {
        ...process.env,
        AMEYA_DEV_DRY_RUN: "1",
        AMEYA_DEV_PORT: "34120",
      },
      encoding: "utf-8",
    });

    expect(run.status).toBe(0);
    const parsed = JSON.parse(run.stdout) as {
      args: string[];
      devUrl: string;
      env: Record<string, string>;
    };
    expect(parsed.devUrl).toBe("http://localhost:34120");
    expect(parsed.args).toContain("tauri");
    expect(parsed.args).toContain("dev");
    expect(parsed.args).toContain("--config");
    expect(parsed.args).toContain("--verbose");
    expect(parsed.env).toMatchObject({
      AMEYA_DEV_PORT: "34120",
      AMEYA_DEV_STRICT_PORT: "1",
    });
  });
});

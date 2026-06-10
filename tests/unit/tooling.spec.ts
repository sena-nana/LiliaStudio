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

  it("includes shell UI dependencies and keeps Ameya state dependencies", () => {
    const pkg = JSON.parse(readWorkspaceFile("package.json"));
    const deps = { ...pkg.dependencies, ...pkg.devDependencies };

    expect(deps.vue).toBeDefined();
    expect(deps["vue-router"]).toBeDefined();
    expect(deps.pinia).toBeDefined();
    expect(deps["@tauri-apps/api"]).toBeDefined();
    expect(deps["@tauri-apps/plugin-store"]).toBeDefined();
    expect(deps["@tauri-apps/plugin-opener"]).toBeDefined();
    expect(deps["@lucide/vue"]).toBeDefined();
    expect(deps.vitepress).toBeDefined();
  });

  it("Rust side keeps Ameya services and window-state plugins", () => {
    const cargo = readWorkspaceFile("src-tauri/Cargo.toml");

    expect(cargo).toContain('tauri-plugin-store = "2.');
    expect(cargo).toContain('tauri-plugin-opener = "2.');
    expect(cargo).toContain("rusqlite");
    expect(cargo).toContain("ureq");
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

  it("GitHub workflows use LiliaStudio naming and project-local paths", () => {
    const ci = readWorkspaceFile(".github/workflows/ci.yml");
    const release = readWorkspaceFile(".github/workflows/release.yml");
    const pages = readWorkspaceFile(".github/workflows/pages.yml");
    const combined = [ci, release, pages].join("\n");

    expect(ci).toContain("corepack yarn verify");
    expect(ci).toContain("corepack yarn docs:build");
    expect(ci).toContain("src-tauri/target");
    expect(release).toContain("projectPath: .");
    expect(release).toContain("releaseName: LiliaStudio");
    expect(release.match(/name: Resolve ref/g)).toHaveLength(1);
    expect(pages).toContain("docs/.vitepress/dist");
    expect(pages).not.toContain("enablement: true");
    expect(combined).not.toContain("apps/desktop");
    expect(combined).not.toContain("LiliaCode");
  });

  it("GitHub Issue templates use LiliaStudio fields without template-only wording", () => {
    const bug = readWorkspaceFile(".github/ISSUE_TEMPLATE/bug_report.yml");
    const feature = readWorkspaceFile(".github/ISSUE_TEMPLATE/feature_request.yml");
    const combined = `${bug}\n${feature}`;

    expect(combined).toContain("LiliaStudio 版本 / commit");
    expect(combined).toContain("构建 / 发布");
    expect(combined).toContain("文档 / GitHub");
    expect(combined).not.toContain("模板版本 / commit");
    expect(combined).not.toContain("Tauri Template");
    expect(combined).not.toContain("LiliaCode");
  });
});

describe("shell styles", () => {
  it("keeps sidebar collapse, resizer, and reduced-motion rules", () => {
    const shellCss = readWorkspaceFile("src/styles/shell.css");

    expect(shellCss).toContain("transition: grid-template-columns 0.24s var(--sidebar-easing)");
    expect(shellCss).toContain("left 0.24s var(--sidebar-easing)");
    expect(shellCss).toContain("@media (prefers-reduced-motion: reduce)");
  });

  it("keeps global transparent button baseline and explicit emphasis states", () => {
    const styles = readWorkspaceFile("src/styles.css");

    expect(styles).toMatch(/button \{\r?\n  background: transparent/);
    expect(styles).toContain("button.primary");
    expect(styles).toContain("background: var(--accent-soft)");
    expect(styles).toContain("button.ghost.danger:hover");
  });
});

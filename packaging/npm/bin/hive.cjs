#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { spawnSync } = require("node:child_process");

const packages = Object.freeze({
  "darwin-arm64": "@aigent-hive/darwin-arm64",
  "darwin-x64": "@aigent-hive/darwin-x64",
  "linux-arm64": "@aigent-hive/linux-arm64",
  "linux-x64": "@aigent-hive/linux-x64",
  "win32-x64": "@aigent-hive/win32-x64",
});

const platformKey = `${process.platform}-${process.arch}`;
const packageName = packages[platformKey];
if (!packageName) {
  process.stderr.write(
    `aigent-hive does not support ${process.platform}/${process.arch}\n`,
  );
  process.exit(4);
}

let packageManifest;
try {
  packageManifest = require.resolve(`${packageName}/package.json`);
} catch {
  process.stderr.write(
    `the native ${packageName} package is missing; reinstall aigent-hive\n`,
  );
  process.exit(5);
}

const executable = path.join(
  path.dirname(packageManifest),
  "bin",
  process.platform === "win32" ? "hive.exe" : "hive",
);
const result = spawnSync(executable, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true,
});
if (result.error) {
  process.stderr.write(`failed to start the native hive binary: ${result.error.message}\n`);
  process.exit(5);
}
if (result.signal) {
  process.stderr.write(`the native hive binary stopped on signal ${result.signal}\n`);
  process.exit(1);
}
process.exit(result.status ?? 1);

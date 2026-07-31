#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const platformDefinitions = Object.freeze({
  "aarch64-apple-darwin": {
    packageName: "@aigent-hive/darwin-arm64",
    directoryName: "darwin-arm64",
    os: "darwin",
    cpu: "arm64",
    executable: "hive",
  },
  "x86_64-apple-darwin": {
    packageName: "@aigent-hive/darwin-x64",
    directoryName: "darwin-x64",
    os: "darwin",
    cpu: "x64",
    executable: "hive",
  },
  "aarch64-unknown-linux-musl": {
    packageName: "@aigent-hive/linux-arm64",
    directoryName: "linux-arm64",
    os: "linux",
    cpu: "arm64",
    executable: "hive",
  },
  "x86_64-unknown-linux-musl": {
    packageName: "@aigent-hive/linux-x64",
    directoryName: "linux-x64",
    os: "linux",
    cpu: "x64",
    executable: "hive",
  },
  "x86_64-pc-windows-msvc": {
    packageName: "@aigent-hive/win32-x64",
    directoryName: "win32-x64",
    os: "win32",
    cpu: "x64",
    executable: "hive.exe",
  },
});
const exactVersionPattern = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/;

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(2);
}

function parseArguments(argv) {
  if (argv.length < 1 || !["platform", "umbrella"].includes(argv[0])) {
    fail("usage: package-npm.mjs platform|umbrella --product-version X.Y.Z --package-version X.Y.Z-test.N --output PATH [--target TRIPLE --binary PATH] [--installer-dir PATH]");
  }
  const options = { kind: argv[0] };
  for (let index = 1; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!flag?.startsWith("--") || value === undefined) {
      fail("npm package arguments must use --name value pairs");
    }
    if (!["--product-version", "--package-version", "--output", "--target", "--binary", "--installer-dir"].includes(flag)) {
      fail(`unsupported npm package argument: ${flag}`);
    }
    options[flag.slice(2)] = value;
  }
  if (!exactVersionPattern.test(options["product-version"] ?? "")) {
    fail("--product-version must be an exact X.Y.Z version");
  }
  const packageVersionPattern = new RegExp(
    `^${options["product-version"].replaceAll(".", "\\.")}-test\\.([1-9][0-9]*)$`,
  );
  if (!packageVersionPattern.test(options["package-version"] ?? "")) {
    fail("--package-version must be PRODUCT_VERSION-test.N with positive N");
  }
  if (!options.output) {
    fail("--output is required");
  }
  if (
    options.kind === "platform"
    && (!options.target || !options.binary || options["installer-dir"])
  ) {
    fail("platform packaging requires --target and --binary");
  }
  if (
    options.kind === "umbrella"
    && (options.target || options.binary || !options["installer-dir"])
  ) {
    fail("umbrella packaging requires --installer-dir and does not accept --target or --binary");
  }
  return options;
}

function requireRegularFile(inputPath, label) {
  const resolved = path.resolve(inputPath);
  let stat;
  try {
    stat = fs.lstatSync(resolved);
  } catch {
    fail(`${label} does not exist`);
  }
  if (!stat.isFile() || stat.isSymbolicLink()) {
    fail(`${label} must be a regular non-symlink file`);
  }
  return resolved;
}

function prepareDirectory(outputRoot, directoryName) {
  const root = path.resolve(outputRoot);
  fs.mkdirSync(root, { recursive: true, mode: 0o755 });
  const destination = path.join(root, directoryName);
  if (fs.existsSync(destination)) {
    fail(`npm package destination already exists: ${destination}`);
  }
  fs.mkdirSync(path.join(destination, "bin"), { recursive: true, mode: 0o755 });
  return destination;
}

function writeJson(destination, value) {
  fs.writeFileSync(destination, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o644,
  });
}

function writePackageReadme(destination, packageName, productVersion, packageVersion) {
  fs.writeFileSync(
    path.join(destination, "README.md"),
    `# ${packageName}\n\nAigent Hive ${productVersion} test package ${packageVersion}. Install the test CLI with \`npm install -g aigent-hive@test\`.\n`,
    { encoding: "utf8", flag: "wx", mode: 0o644 },
  );
}

function commonManifest(name, productVersion, packageVersion) {
  return {
    name,
    version: packageVersion,
    aigentHive: {
      productVersion,
    },
    description: "Provider-neutral local agent harness for subscription-authenticated hosts",
    license: "Apache-2.0",
    repository: {
      type: "git",
      url: "git+https://github.com/gvm1229/aigent-hive.git",
    },
    homepage: "https://github.com/gvm1229/aigent-hive",
    bugs: "https://github.com/gvm1229/aigent-hive/issues",
    publishConfig: {
      access: "public",
      provenance: true,
    },
  };
}

function packagePlatform(options) {
  const definition = platformDefinitions[options.target];
  if (!definition) {
    fail(`unsupported Rust target: ${options.target}`);
  }
  const binary = requireRegularFile(options.binary, "native binary");
  const destination = prepareDirectory(options.output, definition.directoryName);
  const manifest = {
    ...commonManifest(
      definition.packageName,
      options["product-version"],
      options["package-version"],
    ),
    os: [definition.os],
    cpu: [definition.cpu],
    files: ["bin/", "LICENSE", "README.md"],
  };
  writeJson(path.join(destination, "package.json"), manifest);
  writePackageReadme(
    destination,
    definition.packageName,
    options["product-version"],
    options["package-version"],
  );
  fs.copyFileSync(path.join(repositoryRoot, "LICENSE"), path.join(destination, "LICENSE"));
  const packagedBinary = path.join(destination, "bin", definition.executable);
  fs.copyFileSync(binary, packagedBinary, fs.constants.COPYFILE_EXCL);
  fs.chmodSync(packagedBinary, 0o755);
  process.stdout.write(`${destination}\n`);
}

function packageUmbrella(options) {
  const destination = prepareDirectory(options.output, "aigent-hive");
  const installerDirectory = path.resolve(options["installer-dir"]);
  const installers = ["install.sh", "install.ps1", "install.cmd"].map((name) => {
    const installer = requireRegularFile(
      path.join(installerDirectory, name),
      `rendered ${name}`,
    );
    const text = fs.readFileSync(installer, "utf8");
    if (/__AIGENT_HIVE_[A-Z0-9_]+__/.test(text)) {
      fail(`rendered ${name} contains an unresolved marker`);
    }
    return [name, installer];
  });
  const optionalDependencies = Object.fromEntries(
    Object.values(platformDefinitions)
      .map(({ packageName }) => [packageName, options["package-version"]])
      .sort(([left], [right]) => left.localeCompare(right)),
  );
  const manifest = {
    ...commonManifest(
      "aigent-hive",
      options["product-version"],
      options["package-version"],
    ),
    bin: {
      hive: "bin/hive.cjs",
    },
    files: [
      "bin/",
      "install.sh",
      "install.ps1",
      "install.cmd",
      "LICENSE",
      "README.md",
    ],
    engines: {
      node: ">=18",
    },
    optionalDependencies,
  };
  writeJson(path.join(destination, "package.json"), manifest);
  writePackageReadme(
    destination,
    "aigent-hive",
    options["product-version"],
    options["package-version"],
  );
  fs.copyFileSync(path.join(repositoryRoot, "LICENSE"), path.join(destination, "LICENSE"));
  const sourceShim = path.join(repositoryRoot, "packaging", "npm", "bin", "hive.cjs");
  const packagedShim = path.join(destination, "bin", "hive.cjs");
  fs.copyFileSync(sourceShim, packagedShim, fs.constants.COPYFILE_EXCL);
  fs.chmodSync(packagedShim, 0o755);
  for (const [name, source] of installers) {
    const packagedInstaller = path.join(destination, name);
    fs.copyFileSync(source, packagedInstaller, fs.constants.COPYFILE_EXCL);
    fs.chmodSync(packagedInstaller, name === "install.sh" ? 0o755 : 0o644);
  }
  process.stdout.write(`${destination}\n`);
}

const options = parseArguments(process.argv.slice(2));
if (options.kind === "platform") {
  packagePlatform(options);
} else {
  packageUmbrella(options);
}

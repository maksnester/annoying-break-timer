#!/usr/bin/env node
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');

let target = process.argv[2];

if (!target) {
  target = 'patch';
}

const SEMVER = /^\d+\.\d+\.\d+$/;

function bumpVersion(current, part) {
  const [major, minor, patch] = current.split('.').map(Number);
  switch (part) {
    case 'major': return `${major + 1}.0.0`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    default: throw new Error(`Unknown bump type: ${part}`);
  }
}

function resolveVersion(current, target) {
  if (['major', 'minor', 'patch'].includes(target)) {
    return bumpVersion(current, target);
  }
  if (SEMVER.test(target)) {
    return target;
  }
  throw new Error(`Invalid version or bump type: ${target}`);
}

const pkgPath = join(root, 'package.json');
const pkg = JSON.parse(await readFile(pkgPath, 'utf8'));
const previousVersion = pkg.version;
const newVersion = resolveVersion(previousVersion, target);

pkg.version = newVersion;
await writeFile(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);

const lockPath = join(root, 'package-lock.json');
try {
  const lock = JSON.parse(await readFile(lockPath, 'utf8'));
  lock.version = newVersion;
  if (lock.packages && lock.packages['']) {
    lock.packages[''].version = newVersion;
  }
  await writeFile(lockPath, `${JSON.stringify(lock, null, 2)}\n`);
} catch {
  // package-lock.json missing; ignore
}

const cargoPath = join(root, 'src-tauri', 'Cargo.toml');
const cargo = await readFile(cargoPath, 'utf8');
let insidePackage = false;
const newCargo = cargo
  .split('\n')
  .map((line) => {
    if (line.trim() === '[package]') {
      insidePackage = true;
    } else if (insidePackage && /^version\s*=\s*"[^"]*"/.test(line)) {
      insidePackage = false;
      return `version = "${newVersion}"`;
    }
    return line;
  })
  .join('\n');
await writeFile(cargoPath, newCargo);

const tauriConfPath = join(root, 'src-tauri', 'tauri.conf.json');
const tauriConf = JSON.parse(await readFile(tauriConfPath, 'utf8'));
tauriConf.version = newVersion;
await writeFile(tauriConfPath, `${JSON.stringify(tauriConf, null, 2)}\n`);

console.log(`Bumped version from ${previousVersion} to ${newVersion}`);

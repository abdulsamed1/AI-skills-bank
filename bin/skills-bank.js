#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

function platformPackageName() {
  const p = process.platform;
  const a = process.arch;

  if (p === 'win32' && a === 'x64') return 'skills-bank-win32-x64';
  if (p === 'darwin' && a === 'x64') return 'skills-bank-darwin-x64';
  if (p === 'darwin' && a === 'arm64') return 'skills-bank-darwin-arm64';
  if (p === 'linux' && a === 'x64') return 'skills-bank-linux-x64';

  return null;
}

function candidateBinaryNames() {
  const exe = process.platform === 'win32' ? 'skills-bank.exe' : 'skills-bank';
  return [exe, path.join('bin', exe)];
}

function findPackagedBinary(pkgName) {
  if (!pkgName) return null;

  try {
    const pkgJsonPath = require.resolve(`${pkgName}/package.json`);
    const pkgRoot = path.dirname(pkgJsonPath);

    for (const rel of candidateBinaryNames()) {
      const abs = path.join(pkgRoot, rel);
      if (fs.existsSync(abs)) {
        return abs;
      }
    }
  } catch {
    return null;
  }

  return null;
}

function findLocalBinary() {
  const root = process.cwd();
  const exe = process.platform === 'win32' ? 'skills-bank.exe' : 'skills-bank';
  const candidates = [
    path.join(root, 'target', 'release', exe),
    path.join(root, 'target', 'debug', exe),
  ];

  for (const c of candidates) {
    if (fs.existsSync(c)) {
      return c;
    }
  }

  return null;
}

function loadExpectedSha(binaryPath) {
  const envSha = (process.env.SKILL_MANAGE_EXPECTED_SHA256 || '').trim().toLowerCase();
  if (envSha) return envSha;

  const sidecar = `${binaryPath}.sha256`;
  if (fs.existsSync(sidecar)) {
    const content = fs.readFileSync(sidecar, 'utf8').trim();
    if (content) {
      return content.split(/\s+/)[0].toLowerCase();
    }
  }

  return null;
}

function sha256File(filePath) {
  const hash = crypto.createHash('sha256');
  const buf = fs.readFileSync(filePath);
  hash.update(buf);
  return hash.digest('hex').toLowerCase();
}

function verifyBinarySha(binaryPath) {
  const expected = loadExpectedSha(binaryPath);
  if (!expected) {
    return;
  }

  const actual = sha256File(binaryPath);
  if (actual !== expected) {
    console.error('[ERROR] Binary SHA-256 mismatch');
    console.error(`  Expected: ${expected}`);
    console.error(`  Actual:   ${actual}`);
    process.exit(1);
  }
}

function resolveBinary() {
  const envBin = (process.env.SKILL_MANAGE_BIN || '').trim();
  if (envBin && fs.existsSync(envBin)) {
    return envBin;
  }

  const pkg = platformPackageName();
  const packaged = findPackagedBinary(pkg);
  if (packaged) {
    return packaged;
  }

  const local = findLocalBinary();
  if (local) {
    return local;
  }

  return null;
}

function main() {
  const binary = resolveBinary();
  if (!binary) {
    console.error('[ERROR] Could not locate skills-bank binary for this OS/arch.');
    console.error(`  platform=${process.platform} arch=${process.arch}`);
    console.error('  Tried optional platform packages and local target/{release,debug}.');
    process.exit(1);
  }

  verifyBinarySha(binary);

  const args = process.argv.slice(2);
  const child = spawnSync(binary, args, { stdio: 'inherit' });

  if (child.error) {
    console.error(`[ERROR] Failed to execute binary: ${child.error.message}`);
    process.exit(1);
  }

  process.exit(child.status ?? 1);
}

main();

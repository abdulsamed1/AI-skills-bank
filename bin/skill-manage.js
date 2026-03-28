#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { existsSync } from 'node:fs';

const __dirname = dirname(fileURLToPath(import.meta.url));

function getBinaryName() {
    const platform = process.platform;
    const arch = process.arch;

    if (platform === 'win32' && arch === 'x64') {
        return 'skill-manage-win32-x64.exe';
    } else if (platform === 'darwin' && arch === 'x64') {
        return 'skill-manage-darwin-x64';
    } else if (platform === 'darwin' && arch === 'arm64') {
        return 'skill-manage-darwin-arm64';
    } else if (platform === 'linux' && arch === 'x64') {
        return 'skill-manage-linux-x64';
    }

    throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

function findBinary() {
    const binaryName = getBinaryName();
    
    // 1. Try local dev binary (built with cargo)
    const localDevPath = join(__dirname, '..', '..', 'skill-manage', 'target', 'release', 'skill-manage.exe');
    if (existsSync(localDevPath)) {
        return localDevPath;
    }

    // 2. Try optionalDependencies path (post-install)
    const packageBinaryPath = join(__dirname, '..', 'node_modules', `skill-manage-${process.platform}-${process.arch}`, 'bin', binaryName);
    if (existsSync(packageBinaryPath)) {
        return packageBinaryPath;
    }

    throw new Error(`Could not find skill-manage binary. Expected at ${localDevPath} or ${packageBinaryPath}`);
}

try {
    const binaryPath = findBinary();
    const args = process.argv.slice(2);

    const child = spawn(binaryPath, args, {
        stdio: 'inherit',
        shell: false
    });

    child.on('exit', (code) => {
        process.exit(code ?? 0);
    });

    child.on('error', (err) => {
        console.error('Failed to start binary:', err);
        process.exit(1);
    });
} catch (err) {
    console.error(err.message);
    process.exit(1);
}

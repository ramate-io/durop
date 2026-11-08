'use strict';

// Regression for NixOS/nixpkgs#536039: Node 24 + Darwin LLVM 21 corrupts worker
// descriptor tracking, then may close reused libuv descriptors during teardown.
const { Worker, isMainThread } = require('node:worker_threads');
const fs = require('node:fs');
const path = require('node:path');

if (isMainThread) {
  const expected = fs.readFileSync(path.join(__dirname, '.node-version'), 'utf8').trim();
  if (process.versions.node !== expected) {
    throw new Error(`Expected locked Node ${expected}; found ${process.versions.node}`);
  }
  const worker = new Worker(__filename);
  const deadline = setTimeout(() => {
    console.error('Node worker file-descriptor check timed out');
    process.exitCode = 1;
    void worker.terminate();
  }, 10_000);
  worker.once('error', error => {
    console.error(error);
    process.exitCode = 1;
  });
  worker.once('exit', code => {
    clearTimeout(deadline);
    if (code !== 0) {
      console.error(`Node worker file-descriptor check exited with ${code}`);
      process.exitCode = 1;
    } else if (!process.exitCode) {
      console.log(`Node ${expected}: worker file-descriptor tracking passed`);
    }
  });
} else {
  // Fail at the first tracking warning instead of flooding CI with 1,599 copies.
  process.on('warning', warning => {
    if (warning.message.includes('unmanaged mode')) throw warning;
  });
  const buffer = Buffer.alloc(16);
  for (let i = 0; i < 800; i++) {
    const fd = fs.openSync('/dev/null', 'r');
    try {
      fs.readSync(fd, buffer, 0, buffer.length, 0);
    } finally {
      fs.closeSync(fd);
    }
  }
}

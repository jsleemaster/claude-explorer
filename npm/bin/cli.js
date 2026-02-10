#!/usr/bin/env node

"use strict";

const { execFileSync } = require("child_process");
const { getBinaryPath } = require("../lib/platform");

const binaryPath = getBinaryPath();

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  if (err.status !== null) {
    process.exit(err.status);
  }
  console.error(`Failed to run cltree: ${err.message}`);
  console.error(`Binary path: ${binaryPath}`);
  process.exit(1);
}

"use strict";

const os = require("os");
const path = require("path");

const PLATFORM_MAP = {
  "darwin-arm64": "cltree-macos-aarch64",
  "darwin-x64": "cltree-macos-x86_64",
  "linux-x64": "cltree-linux-x86_64",
  "linux-arm64": "cltree-linux-aarch64",
  "win32-x64": "cltree-windows-x86_64.exe",
};

function getPlatformKey() {
  return `${os.platform()}-${os.arch()}`;
}

function getAssetName() {
  const key = getPlatformKey();
  const asset = PLATFORM_MAP[key];
  if (!asset) {
    throw new Error(
      `Unsupported platform: ${key}\n` +
        `Supported platforms: ${Object.keys(PLATFORM_MAP).join(", ")}`
    );
  }
  return asset;
}

function getBinaryName() {
  return os.platform() === "win32" ? "cltree.exe" : "cltree";
}

function getBinaryPath() {
  return path.join(__dirname, "..", "bin", getBinaryName());
}

function getDownloadUrl(version) {
  const asset = getAssetName();
  return `https://github.com/jsleemaster/cltree/releases/download/v${version}/${asset}`;
}

module.exports = { getAssetName, getBinaryName, getBinaryPath, getDownloadUrl, getPlatformKey };

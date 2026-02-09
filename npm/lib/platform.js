"use strict";

const os = require("os");
const path = require("path");

const PLATFORM_MAP = {
  "darwin-arm64": "claude-explorer-macos-aarch64",
  "darwin-x64": "claude-explorer-macos-x86_64",
  "linux-x64": "claude-explorer-linux-x86_64",
  "linux-arm64": "claude-explorer-linux-aarch64",
  "win32-x64": "claude-explorer-windows-x86_64.exe",
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
  return os.platform() === "win32" ? "claude-explorer.exe" : "claude-explorer";
}

function getBinaryPath() {
  return path.join(__dirname, "..", "bin", getBinaryName());
}

function getDownloadUrl(version) {
  const asset = getAssetName();
  return `https://github.com/jsleemaster/claude-explorer/releases/download/v${version}/${asset}`;
}

module.exports = { getAssetName, getBinaryName, getBinaryPath, getDownloadUrl, getPlatformKey };

"use strict";

const https = require("https");
const fs = require("fs");
const path = require("path");
const { getBinaryPath, getDownloadUrl, getPlatformKey } = require("./lib/platform");

const MAX_REDIRECTS = 5;

function main() {
  if (process.env.CLAUDE_EXPLORER_SKIP_INSTALL) {
    console.log("Skipping binary download (CLAUDE_EXPLORER_SKIP_INSTALL is set)");
    return;
  }

  const pkg = require("./package.json");
  const version = pkg.version;
  const url = getDownloadUrl(version);
  const dest = getBinaryPath();

  // Ensure bin directory exists
  fs.mkdirSync(path.dirname(dest), { recursive: true });

  console.log(`Downloading claude-explorer v${version} for ${getPlatformKey()}...`);
  console.log(`  ${url}`);

  download(url, dest, 0)
    .then(() => {
      // Make binary executable on Unix
      if (process.platform !== "win32") {
        fs.chmodSync(dest, 0o755);
      }
      console.log("claude-explorer installed successfully!");
    })
    .catch((err) => {
      console.error(`\nFailed to download claude-explorer: ${err.message}\n`);
      console.error("You can install manually:");
      console.error(`  cargo install claude-explorer`);
      console.error(`  Or download from: https://github.com/jsleemaster/claude-explorer/releases`);
      process.exit(1);
    });
}

function download(url, dest, redirectCount) {
  return new Promise((resolve, reject) => {
    if (redirectCount > MAX_REDIRECTS) {
      reject(new Error("Too many redirects"));
      return;
    }

    const parsedUrl = new URL(url);
    const options = {
      hostname: parsedUrl.hostname,
      path: parsedUrl.pathname + parsedUrl.search,
      headers: { "User-Agent": "claude-explorer-npm-installer" },
    };

    https
      .get(options, (res) => {
        // Follow redirects
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          return resolve(download(res.headers.location, dest, redirectCount + 1));
        }

        if (res.statusCode !== 200) {
          res.resume();
          reject(new Error(`HTTP ${res.statusCode}: ${url}`));
          return;
        }

        const totalBytes = parseInt(res.headers["content-length"], 10) || 0;
        let downloadedBytes = 0;

        const file = fs.createWriteStream(dest);

        res.on("data", (chunk) => {
          downloadedBytes += chunk.length;
          if (totalBytes > 0) {
            const pct = ((downloadedBytes / totalBytes) * 100).toFixed(1);
            const mb = (downloadedBytes / 1024 / 1024).toFixed(1);
            const totalMb = (totalBytes / 1024 / 1024).toFixed(1);
            process.stdout.write(`\r  ${mb}MB / ${totalMb}MB (${pct}%)`);
          }
        });

        res.pipe(file);

        file.on("finish", () => {
          if (totalBytes > 0) process.stdout.write("\n");
          file.close(resolve);
        });

        file.on("error", (err) => {
          fs.unlink(dest, () => {});
          reject(err);
        });
      })
      .on("error", reject);
  });
}

main();

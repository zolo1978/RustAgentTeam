// check-for-update.ts — Frontend auto-update for Tauri v2
// Usage: Copy into your frontend src/ and call checkForUpdate() on app startup

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

async function checkForUpdate(opts?: { requireConfirmation?: boolean }) {
  try {
    const update = await check();
    if (!update?.available) return;

    // Show update details and ask for user confirmation before proceeding.
    if (opts?.requireConfirmation !== false) {
      const version = update.version ?? "unknown";
      const body = update.body ?? "";
      const confirmed = window.confirm(
        `A new version (${version}) is available.\n\n${body}\n\nDownload and install now?`
      );
      if (!confirmed) return;
    }

    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          console.log(`Downloading ${event.data.contentLength} bytes`);
          break;
        case "Progress":
          console.log(`Downloaded ${event.data.chunkLength} bytes`);
          break;
        case "Finished":
          console.log("Download complete, installing...");
          break;
      }
    });

    await relaunch();
  } catch (err) {
    console.error("Update failed:", err);
    // Don't block user — retry on next launch
  }
}

export { checkForUpdate };

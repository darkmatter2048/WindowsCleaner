import { spawn } from "node:child_process";

function killProcessImage(imageName) {
  return new Promise((resolve) => {
    const child = spawn("taskkill", ["/IM", imageName, "/F"], {
      stdio: "ignore",
      shell: false,
    });

    child.on("error", () => resolve());
    child.on("exit", () => resolve());
  });
}

if (process.platform === "win32") {
  await killProcessImage("wc.exe");
  await killProcessImage("node.exe");
}

console.log("[dev-reset] cleaned wc.exe and node.exe");

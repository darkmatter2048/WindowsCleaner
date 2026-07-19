import { spawn } from "node:child_process";
import { createServer } from "vite";

const PORT = 1420;
const HOST = process.env.TAURI_DEV_HOST || "127.0.0.1";
const READY_URL = `http://${HOST}:${PORT}/`;

function keepProcessAlive() {
  setInterval(() => {}, 60 * 60 * 1000);
  process.stdin.resume();
}

async function isServerReady(url) {
  try {
    const response = await fetch(url, { method: "HEAD" });
    return response.ok || response.status === 404;
  } catch {
    return false;
  }
}

async function waitForServer(url, timeoutMs = 15_000) {
  const startedAt = Date.now();

  while (Date.now() - startedAt < timeoutMs) {
    if (await isServerReady(url)) return;
    await new Promise((resolve) => setTimeout(resolve, 150));
  }

  throw new Error(`Vite dev server did not become ready at ${url}`);
}

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

async function ensureCleanTauriBinary() {
  if (process.platform !== "win32") return;
  await killProcessImage("wc.exe");
}

let server;

await ensureCleanTauriBinary();

if (await isServerReady(READY_URL)) {
  console.log(`[dev-server] reusing existing server: ${READY_URL}`);
  keepProcessAlive();
} else {
  try {
    server = await createServer({
      configFile: "vite.config.ts",
      server: {
        host: HOST,
        port: PORT,
        strictPort: true,
      },
    });

    await server.listen();
  } catch (error) {
    if (await isServerReady(READY_URL)) {
      console.log(`[dev-server] port was busy, but ${READY_URL} is ready; reusing it.`);
      keepProcessAlive();
    } else {
      throw error;
    }
  }

  if (server) {
    server.printUrls();
    await waitForServer(READY_URL);
    console.log(`[dev-server] ready: ${READY_URL}`);
    keepProcessAlive();
  }
}

const close = async () => {
  if (server) {
    await server.close();
  }
  process.exit(0);
};

process.on("SIGINT", close);
process.on("SIGTERM", close);

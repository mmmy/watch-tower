const { spawn } = require("node:child_process");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const args = ["test", "--manifest-path", "native-shell/Cargo.toml", ...process.argv.slice(2)];

const child = spawn("cargo", args, {
  cwd: repoRoot,
  env: {
    ...process.env,
    CARGO_TARGET_DIR: path.join(repoRoot, "native-shell", "target-tests"),
  },
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 0);
});

child.on("error", (error) => {
  console.error(error.message);
  process.exit(1);
});

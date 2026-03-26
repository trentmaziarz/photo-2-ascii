const { execSync, execFileSync } = require("child_process");
const os = require("os");
const path = require("path");
const fs = require("fs");

const ROOT = path.resolve(__dirname, "..");
const isWindows = os.platform() === "win32";

function run(cmd, opts = {}) {
  console.log(`> ${cmd}`);
  execSync(cmd, { stdio: "inherit", cwd: ROOT, shell: true, ...opts });
}

function hasCargo() {
  try {
    execFileSync("cargo", ["--version"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function hasRustup() {
  try {
    execFileSync("rustup", ["--version"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function installRust() {
  console.log("Rust toolchain not found. Installing via rustup...\n");

  if (isWindows) {
    const installerUrl = "https://win.rustup.rs/x86_64";
    const installerPath = path.join(os.tmpdir(), "rustup-init.exe");

    // Download rustup-init.exe using PowerShell
    run(
      `powershell -Command "Invoke-WebRequest -Uri '${installerUrl}' -OutFile '${installerPath}'"`,
    );
    // Run installer with defaults (unattended)
    run(`"${installerPath}" -y`);
  } else {
    // Unix: curl | sh
    run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y");
  }

  // Add cargo to PATH for this process
  const cargoBin = path.join(os.homedir(), ".cargo", "bin");
  process.env.PATH = cargoBin + path.delimiter + process.env.PATH;

  if (!hasCargo()) {
    console.error(
      "\nFailed to install Rust. Please install manually: https://rustup.rs",
    );
    process.exit(1);
  }

  console.log("Rust installed successfully.\n");
}

// --- Main ---

if (!hasCargo()) {
  if (hasRustup()) {
    console.log("rustup found but no default toolchain. Installing stable...\n");
    run("rustup default stable");
  } else {
    installRust();
  }
}

// Show versions
run("rustc --version");
run("cargo --version");

// Build release
console.log("\nBuilding ascii-artist (release)...\n");
run("cargo build --release");

const ext = isWindows ? ".exe" : "";
const binary = path.join(ROOT, "target", "release", `ascii-artist${ext}`);

if (fs.existsSync(binary)) {
  const size = (fs.statSync(binary).size / (1024 * 1024)).toFixed(1);
  console.log(`\nBuild complete: ${binary} (${size} MB)`);
} else {
  console.error("\nBuild failed: binary not found.");
  process.exit(1);
}

#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');
const https = require('https');
const http = require('http');

const REPO = 'relharrati/pawlos-ai-agent';
const BRANCH = 'master';

const colors = {
  reset: '\x1b[0m',
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  magenta: '\x1b[35m',
  bold: '\x1b[1m'
};

function log(msg, color = 'cyan') {
  console.log(`${colors[color]}${msg}${colors.reset}`);
}

function info(msg) { log(`[pawlos] ${msg}`, 'cyan'); }
function ok(msg)   { log(`[  ok  ] ${msg}`, 'green'); }
function err(msg)  { log(`[ err  ] ${msg}`, 'red'); process.exit(1); }

// Print banner
function printBanner() {
  console.log(`${colors.magenta}`);
  console.log(`  ____    _             _           _           `);
  console.log(` |  _ \\  | |           | |         | |         `);
  console.log(` | | | | | |_  _   _  | |_  _   _ | |_        `);
  console.log(` | | | | | __|| | | | | __|| | | || __|       `);
  console.log(` | |_| | | |_ | |_| | | |_ | |_| || |_        `);
  console.log(`  \\___/   \\__| \\__,_|  \\__| \\__,_| \\__|       `);
  console.log(`${colors.reset}`);
}

// Download file helper
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const protocol = url.startsWith('https') ? https : http;
    protocol.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        file.close();
        download(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      response.pipe(file);
      file.on('finish', () => { file.close(); resolve(); });
    }).on('error', (e) => { file.close(); reject(e); });
  });
}

// Install dependencies (MCP servers, etc.)
async function installDeps() {
  info('Installing MCP dependencies...');
  
  const { execSync } = require('child_process');
  
  try {
    // Check if npm available
    execSync('npm --version', { stdio: 'ignore' });
    
    // Install common MCP packages
    const packages = [
      '@modelcontextprotocol/server-filesystem',
      '@modelcontextprotocol/server-fetch',
      '@modelcontextprotocol/server-github'
    ];
    
    for (const pkg of packages) {
      try {
        execSync(`npm install -g ${pkg}`, { stdio: 'ignore' });
      } catch (e) {
        // Ignore failures, continue
      }
    }
    
    ok('Dependencies installed');
  } catch (e) {
    // Not critical
  }
}

// Run pawlos CLI
async function main() {
  const args = process.argv.slice(2);
  
  // Handle special commands
  if (args[0] === 'install-deps') {
    await installDeps();
    return;
  }
  
  if (args[0] === 'onboard' || args[0] === 'install') {
    printBanner();
    console.log('');
    // Run the installer
    const platform = os.platform();
    const isWin = platform === 'win32';
    
    if (isWin) {
      info('Use PowerShell:');
      console.log(`${colors.green}  iwr -useb https://raw.githubusercontent.com/${REPO}/${BRANCH}/scripts/install.ps1 | iex${colors.reset}`);
    } else {
      info('Use curl:');
      console.log(`${colors.green}  curl -sSL https://raw.githubusercontent.com/${REPO}/${BRANCH}/scripts/install.sh | bash${colors.reset}`);
    }
    return;
  }
  
  // Check if pawlos is installed
  const isWin = os.platform() === 'win32';
  const pawlosPath = path.join(os.homedir(), '.pawlos', 'pawlos');
  const binPath = isWin ? pawlosPath + '.exe' : pawlosPath;
  
  // Alternative paths
  const altPaths = isWin 
    ? [path.join(os.homedir(), 'AppData', 'Local', 'Programs', 'pawlos', 'pawlos.exe')]
    : [path.join(os.homedir(), '.local', 'bin', 'pawlos'), '/usr/local/bin/pawlos'];
  
  let foundPath = null;
  for (const p of [binPath, ...altPaths]) {
    if (fs.existsSync(p)) {
      foundPath = p;
      break;
    }
  }
  
  if (!foundPath) {
    printBanner();
    console.log('');
    err('pawlos not found. Install with:');
    console.log('');
    const platform = os.platform();
    if (platform === 'win32') {
      console.log(`  ${colors.green}iwr -useb https://raw.githubusercontent.com/${REPO}/${BRANCH}/scripts/install.ps1 | iex${colors.reset}`);
    } else {
      console.log(`  ${colors.green}curl -sSL https://raw.githubusercontent.com/${REPO}/${BRANCH}/scripts/install.sh | bash${colors.reset}`);
    }
    console.log('');
    console.log(`  Or via npx: ${colors.green}npx https://raw.githubusercontent.com/${REPO}/${BRANCH}/scripts/bin/pawlos.js${colors.reset}`);
    console.log('');
    return;
  }
  
  // Run pawlos
  const child = spawn(foundPath, args, {
    stdio: 'inherit',
    shell: isWin
  });
  
  child.on('exit', (code) => {
    process.exit(code || 0);
  });
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
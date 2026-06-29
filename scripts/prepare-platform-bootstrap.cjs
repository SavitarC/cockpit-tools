#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const DEFAULT_OUTPUT_DIR = path.join(ROOT, 'platform-packages', 'bootstrap');

function fail(message) {
  console.error(message);
  process.exit(1);
}

function usage() {
  console.log(`Usage:
  node scripts/prepare-platform-bootstrap.cjs --index-url <url> --targets <os/arch,...> [options]

Options:
  --index-file <path>       Use a local platform package index instead of --index-url.
  --output-dir <path>       Output bootstrap dir. Defaults to platform-packages/bootstrap.
  --targets <list>          Comma list, for example macos/aarch64,macos/x86_64.
`);
}

function parseArgs(argv) {
  const args = {
    outputDir: DEFAULT_OUTPUT_DIR,
    targets: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      usage();
      process.exit(0);
    }
    const next = argv[index + 1];
    if (!next || next.startsWith('--')) fail(`Missing value for ${arg}`);
    index += 1;
    if (arg === '--index-url') args.indexUrl = next;
    else if (arg === '--index-file') args.indexFile = path.resolve(ROOT, next);
    else if (arg === '--output-dir') args.outputDir = path.resolve(ROOT, next);
    else if (arg === '--targets') args.targets = parseTargets(next);
    else fail(`Unknown argument: ${arg}`);
  }
  if (!args.indexUrl && !args.indexFile) fail('Missing --index-url or --index-file');
  if (args.targets.length === 0) fail('Missing --targets <os/arch,...>');
  return args;
}

function parseTargets(value) {
  return String(value || '')
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => {
      const [os, arch] = item.split('/');
      if (!os || !arch) fail(`Invalid target: ${item}`);
      return { os, arch, key: `${os}/${arch}` };
    });
}

function readJson(filePath, label) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    fail(`${label}: failed to read JSON: ${error.message}`);
  }
}

async function readRemoteJson(url) {
  const response = await fetch(url, {
    headers: {
      'Cache-Control': 'no-cache',
      Pragma: 'no-cache',
      'User-Agent': 'Cockpit-Tools-Bootstrap',
    },
  });
  if (!response.ok) fail(`Failed to download index: HTTP ${response.status} ${url}`);
  return await response.json();
}

function sha256Buffer(buffer) {
  return crypto.createHash('sha256').update(buffer).digest('hex');
}

function zipNameFromUrl(url) {
  const parsed = new URL(url);
  const name = path.basename(parsed.pathname);
  if (!name || !name.endsWith('.zip') || name.includes('/') || name.includes('\\')) {
    fail(`Invalid artifact zip name from ${url}`);
  }
  return name;
}

async function downloadArtifact(artifact, distDir) {
  const zipName = zipNameFromUrl(artifact.downloadUrl);
  const zipPath = path.join(distDir, zipName);
  const response = await fetch(artifact.downloadUrl, {
    headers: { 'User-Agent': 'Cockpit-Tools-Bootstrap' },
  });
  if (!response.ok) {
    fail(`Failed to download ${artifact.downloadUrl}: HTTP ${response.status}`);
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  if (artifact.downloadSizeBytes && bytes.length !== artifact.downloadSizeBytes) {
    fail(`${zipName}: size mismatch, expected ${artifact.downloadSizeBytes}, actual ${bytes.length}`);
  }
  const actualSha = sha256Buffer(bytes);
  if (actualSha !== artifact.sha256) {
    fail(`${zipName}: sha256 mismatch, expected ${artifact.sha256}, actual ${actualSha}`);
  }
  fs.writeFileSync(zipPath, bytes);
  return {
    ...artifact,
    downloadUrl: `https://bootstrap.local/${zipName}`,
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const sourceIndex = args.indexFile
    ? readJson(args.indexFile, args.indexFile)
    : await readRemoteJson(args.indexUrl);
  const targetKeys = new Set(args.targets.map((target) => target.key));
  const distDir = path.join(args.outputDir, 'dist');
  fs.rmSync(args.outputDir, { recursive: true, force: true });
  fs.mkdirSync(distDir, { recursive: true });

  const packages = [];
  for (const pkg of sourceIndex.packages || []) {
    const artifacts = [];
    for (const target of args.targets) {
      const artifact = (pkg.artifacts || []).find(
        (item) => item.os === target.os && item.arch === target.arch,
      );
      if (!artifact) {
        fail(`${pkg.id}: missing artifact for ${target.key}`);
      }
      artifacts.push(await downloadArtifact(artifact, distDir));
    }
    const uniqueArtifacts = artifacts.filter((artifact, index, list) => (
      list.findIndex((item) => `${item.os}/${item.arch}` === `${artifact.os}/${artifact.arch}`) === index
    ));
    const primary = uniqueArtifacts.find((artifact) => targetKeys.has(`${artifact.os}/${artifact.arch}`))
      || uniqueArtifacts[0];
    packages.push({
      ...pkg,
      artifacts: uniqueArtifacts,
      downloadUrl: primary.downloadUrl,
      downloadSizeBytes: primary.downloadSizeBytes,
      sha256: primary.sha256,
    });
  }

  const outputIndex = {
    ...sourceIndex,
    version: sourceIndex.version || 'bootstrap',
    packages,
  };
  const outputIndexPath = path.join(args.outputDir, 'index.json');
  fs.writeFileSync(outputIndexPath, `${JSON.stringify(outputIndex, null, 2)}\n`);
  console.log(`Prepared platform bootstrap for targets: ${[...targetKeys].join(', ')}`);
  console.log(`Wrote ${path.relative(ROOT, outputIndexPath)}`);
}

main().catch((error) => fail(error?.message || String(error)));

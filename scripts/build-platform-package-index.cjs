#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const DEFAULT_BASE_INDEX = path.join(ROOT, 'platform-packages', 'index.json');
const DEFAULT_METADATA_DIR = path.join(ROOT, 'platform-packages', 'dist-ci');
const DEFAULT_OUTPUT = path.join(ROOT, 'platform-packages', 'dist-ci', 'index.json');
const DEFAULT_ORDER = [
  'macos/aarch64',
  'macos/x86_64',
  'linux/x86_64',
  'linux/aarch64',
  'windows/x86_64',
];

function fail(message) {
  console.error(message);
  process.exit(1);
}

function usage() {
  console.log(`Usage:
  node scripts/build-platform-package-index.cjs [options]

Options:
  --metadata-dir <path>          Directory containing artifact metadata JSON files.
  --base-index <path>            Base platform-packages/index.json.
  --output <path>                Output merged index JSON.
  --download-base-url <url>      Override artifact downloadUrl with <url>/<zipName>.
  --require-os-arch <list>       Comma list such as macos/aarch64,linux/x86_64.
  --verify-zip-dir <path>        Verify zip size and sha256 against metadata.
`);
}

function parseArgs(argv) {
  const args = {
    metadataDir: DEFAULT_METADATA_DIR,
    baseIndex: DEFAULT_BASE_INDEX,
    output: DEFAULT_OUTPUT,
    requiredTargets: [],
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
    if (arg === '--metadata-dir') args.metadataDir = path.resolve(ROOT, next);
    else if (arg === '--base-index') args.baseIndex = path.resolve(ROOT, next);
    else if (arg === '--output') args.output = path.resolve(ROOT, next);
    else if (arg === '--download-base-url') args.downloadBaseUrl = next.replace(/\/+$/, '');
    else if (arg === '--require-os-arch') args.requiredTargets = parseTargets(next);
    else if (arg === '--verify-zip-dir') args.verifyZipDir = path.resolve(ROOT, next);
    else fail(`Unknown argument: ${arg}`);
  }

  return args;
}

function parseTargets(value) {
  return String(value || '')
    .split(',')
    .map((target) => target.trim())
    .filter(Boolean)
    .map((target) => {
      const [os, arch] = target.split('/');
      if (!os || !arch) fail(`Invalid --require-os-arch target: ${target}`);
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

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function displayPath(filePath) {
  const relativePath = path.relative(ROOT, filePath);
  if (relativePath === '' || (!relativePath.startsWith('..') && !path.isAbsolute(relativePath))) {
    return relativePath;
  }
  return filePath;
}

function walkJsonFiles(dir) {
  if (!fs.existsSync(dir) || !fs.statSync(dir).isDirectory()) {
    fail(`metadata dir does not exist: ${dir}`);
  }

  const files = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...walkJsonFiles(entryPath));
    else if (entry.isFile() && entry.name.endsWith('.json')) files.push(entryPath);
  }
  return files;
}

function isArtifactMetadata(value) {
  return value
    && typeof value === 'object'
    && typeof value.id === 'string'
    && typeof value.version === 'string'
    && typeof value.os === 'string'
    && typeof value.arch === 'string'
    && typeof value.zipName === 'string'
    && typeof value.downloadSizeBytes === 'number'
    && typeof value.sha256 === 'string';
}

function targetSortKey(artifact) {
  const key = `${artifact.os}/${artifact.arch}`;
  const knownIndex = DEFAULT_ORDER.indexOf(key);
  if (knownIndex >= 0) return `${String(knownIndex).padStart(3, '0')}:${key}`;
  return `999:${key}`;
}

function verifyMetadataZip(metadata, metadataFile, verifyZipDir) {
  if (!verifyZipDir) return;
  const zipPath = path.join(verifyZipDir, metadata.zipName);
  if (!fs.existsSync(zipPath)) {
    fail(`${path.relative(ROOT, metadataFile)}: missing zip ${path.relative(ROOT, zipPath)}`);
  }
  const actualSize = fs.statSync(zipPath).size;
  const actualSha = sha256(zipPath);
  if (actualSize !== metadata.downloadSizeBytes) {
    fail(`${metadata.id} ${metadata.os}/${metadata.arch}: zip size mismatch`);
  }
  if (actualSha !== metadata.sha256) {
    fail(`${metadata.id} ${metadata.os}/${metadata.arch}: zip sha256 mismatch`);
  }
}

function artifactFromMetadata(metadata, downloadBaseUrl) {
  return {
    os: metadata.os,
    arch: metadata.arch,
    downloadUrl: downloadBaseUrl ? `${downloadBaseUrl}/${metadata.zipName}` : metadata.downloadUrl,
    downloadSizeBytes: metadata.downloadSizeBytes,
    sha256: metadata.sha256,
  };
}

function collectMetadata(args) {
  const byPackage = new Map();
  for (const filePath of walkJsonFiles(args.metadataDir)) {
    const metadata = readJson(filePath, path.relative(ROOT, filePath));
    if (!isArtifactMetadata(metadata)) continue;
    verifyMetadataZip(metadata, filePath, args.verifyZipDir);

    const packageMap = byPackage.get(metadata.id) || new Map();
    const targetKey = `${metadata.os}/${metadata.arch}`;
    if (packageMap.has(targetKey)) {
      fail(`${metadata.id}: duplicate metadata for ${targetKey}`);
    }
    packageMap.set(targetKey, { metadata, filePath });
    byPackage.set(metadata.id, packageMap);
  }
  return byPackage;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const baseIndex = readJson(args.baseIndex, 'base platform package index');
  const byPackage = collectMetadata(args);
  const rows = [];

  const merged = {
    ...baseIndex,
    packages: (baseIndex.packages || []).map((pkg) => {
      const packageMap = byPackage.get(pkg.id);
      if (!packageMap) fail(`${pkg.id}: missing artifact metadata`);
      if (args.requiredTargets.length > 0) {
        for (const target of args.requiredTargets) {
          if (!packageMap.has(target.key)) {
            fail(`${pkg.id}: missing artifact metadata for ${target.key}`);
          }
        }
      }

      const artifacts = [...packageMap.values()]
        .map(({ metadata }) => {
          if (metadata.version !== pkg.version) fail(`${pkg.id}: metadata version mismatch`);
          if (metadata.platformId !== pkg.platformId) fail(`${pkg.id}: metadata platformId mismatch`);
          if (metadata.packageMode !== pkg.packageMode) fail(`${pkg.id}: metadata packageMode mismatch`);
          if (metadata.installKind !== pkg.installKind) fail(`${pkg.id}: metadata installKind mismatch`);
          return artifactFromMetadata(metadata, args.downloadBaseUrl);
        })
        .sort((left, right) => targetSortKey(left).localeCompare(targetSortKey(right)));

      const primaryArtifact = artifacts[0];
      rows.push({
        id: pkg.id,
        version: pkg.version,
        artifacts: artifacts.length,
        primary: `${primaryArtifact.os}/${primaryArtifact.arch}`,
      });

      return {
        ...pkg,
        artifacts,
        downloadUrl: primaryArtifact.downloadUrl,
        downloadSizeBytes: primaryArtifact.downloadSizeBytes,
        sha256: primaryArtifact.sha256,
      };
    }),
  };

  fs.mkdirSync(path.dirname(args.output), { recursive: true });
  fs.writeFileSync(args.output, `${JSON.stringify(merged, null, 2)}\n`);
  console.table(rows);
  console.log(`Wrote merged platform package index -> ${displayPath(args.output)}`);
}

main();

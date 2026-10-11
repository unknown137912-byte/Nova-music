import { execFileSync } from 'node:child_process';
import { mkdir, readdir, rm, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const target = process.env.NOVA_TARGET_TRIPLE || '';
if (target && target !== 'aarch64-apple-darwin') {
  throw new Error(`NOVA supports Apple Silicon only; unsupported macOS target: ${target}.`);
}
if (!target && process.arch !== 'arm64') {
  throw new Error(`NOVA supports Apple Silicon only; current architecture is ${process.arch}.`);
}
const targetDir = target ? join(root, 'target', target, 'release') : join(root, 'target', 'release');
const binary = join(targetDir, 'nova-music');
const brewPrefix = execFileSync('brew', ['--prefix'], { encoding: 'utf8' }).trim();
const architecture = 'arm64';
const librariesDir = join(root, 'target', 'macos-libraries', architecture);
const bundleConfig = join(root, 'target', `tauri-bundle-${architecture}.json`);

function run(command, args, options = {}) {
  try {
    execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], ...options });
  } catch (error) {
    if (error.stdout) process.stderr.write(error.stdout);
    if (error.stderr) process.stderr.write(error.stderr);
    throw error;
  }
}

try {
  execFileSync('test', ['-x', binary]);
} catch {
  throw new Error(`Built NOVA executable was not found at ${binary}. Run cargo tauri build --no-bundle first.`);
}

const binaryLibraries = execFileSync('otool', ['-L', binary], { encoding: 'utf8' });
if (/@executable_path\/\.\.\/Frameworks\/libmpv/.test(binaryLibraries)) {
  throw new Error(`The executable at ${binary} was already bundled. Rebuild it with cargo tauri build --no-bundle before preparing libraries again.`);
}

await rm(librariesDir, { recursive: true, force: true });
await mkdir(librariesDir, { recursive: true });

run('dylibbundler', [
  '-cd', '-of', '-b',
  '-x', binary,
  '-d', librariesDir,
  '-p', '@executable_path/../Frameworks',
  '-s', join(brewPrefix, 'lib'),
]);

const libraries = (await readdir(librariesDir))
  .filter((name) => name.endsWith('.dylib'))
  .sort()
  .map((name) => join(librariesDir, name));
if (!libraries.some((path) => path.includes('libmpv'))) {
  throw new Error('Bundling did not include libmpv; the app would not play audio on a clean Mac.');
}

const machOFiles = [binary, ...libraries];
for (const file of machOFiles) {
  const loadCommands = execFileSync('otool', ['-L', file], { encoding: 'utf8' });
  const badLoadPath = loadCommands.split('\n').find((line) => /\/(?:opt\/homebrew|usr\/local\/(?:Cellar|opt))\//.test(line));
  if (badLoadPath) throw new Error(`Unbundled Homebrew dependency in ${file}: ${badLoadPath.trim()}`);

  const rpathOutput = execFileSync('otool', ['-l', file], { encoding: 'utf8' });
  const paths = [...rpathOutput.matchAll(/cmd LC_RPATH\s+cmdsize \d+\s+path (.+?) \(offset \d+\)/g)].map((match) => match[1]);
  const counts = new Map();
  for (const path of paths) counts.set(path, (counts.get(path) || 0) + 1);
  for (const [path, count] of counts) {
    for (let duplicate = 1; duplicate < count; duplicate += 1) {
      run('install_name_tool', ['-delete_rpath', path, file]);
    }
  }
}

const hasDeveloperIdentity = Boolean(process.env.APPLE_SIGNING_IDENTITY && process.env.APPLE_SIGNING_IDENTITY !== '-');
const config = {
  bundle: {
    macOS: {
      frameworks: libraries,
      hardenedRuntime: hasDeveloperIdentity,
    },
  },
};
await writeFile(bundleConfig, `${JSON.stringify(config, null, 2)}\n`);
console.log(`Prepared ${libraries.length} macOS libraries for ${architecture}; hardened runtime ${hasDeveloperIdentity ? 'enabled' : 'disabled'}; wrote ${bundleConfig}.`);

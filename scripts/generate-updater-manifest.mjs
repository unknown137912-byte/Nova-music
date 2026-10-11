import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const repository = process.env.GITHUB_REPOSITORY;
const tag = process.env.GITHUB_REF_NAME;
if (!repository || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
  throw new Error('GITHUB_REPOSITORY must be set to owner/repository.');
}
if (!tag) throw new Error('GITHUB_REF_NAME must contain the release tag.');

const releaseDirectory = join(process.cwd(), 'release-assets');
const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
const version = config.version;
const platforms = {};

for (const [arch, target] of [['arm64', 'darwin-aarch64'], ['intel', 'darwin-x86_64']]) {
  const baseName = `NOVA-Music-${version}-macos-${arch}.app.tar.gz`;
  const archive = join(releaseDirectory, baseName);
  const signature = await readFile(`${archive}.sig`, 'utf8');
  platforms[target] = {
    url: `https://github.com/${repository}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(baseName)}`,
    signature: signature.trim(),
  };
}

const manifest = {
  version,
  notes: `NOVA Music ${version} for macOS.`,
  pub_date: new Date().toISOString(),
  platforms,
};
await writeFile(join(releaseDirectory, 'latest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Generated signed update manifest for ${repository} (${version}).`);

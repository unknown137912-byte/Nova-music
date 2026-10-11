import { readFile, writeFile } from 'node:fs/promises';

const repository = process.env.GITHUB_REPOSITORY;
if (!repository || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
  throw new Error('GITHUB_REPOSITORY must be set to owner/repository by GitHub Actions.');
}

const configPath = new URL('../src-tauri/tauri.conf.json', import.meta.url);
const config = JSON.parse(await readFile(configPath, 'utf8'));
config.plugins.updater.endpoints = [
  `https://github.com/${repository}/releases/latest/download/latest.json`,
];
await writeFile(configPath, `${JSON.stringify(config, null, 2)}\n`);
console.log(`Configured updater feed for ${repository}.`);

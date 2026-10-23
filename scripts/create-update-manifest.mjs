import { readFileSync, writeFileSync, statSync } from 'node:fs';
import { join, basename } from 'node:path';
import { pathToFileURL } from 'node:url';

// Both build jobs must finish before generating a manifest. A manifest that
// refers to a missing platform would break the updater for all platforms.
export function createManifest(directory, tag, repository, now = new Date()) {
  if (!/^v\d+\.\d+\.\d+$/.test(tag)) throw new Error('Updates require a stable vX.Y.Z release tag');
  if (!/^[\w.-]+\/[\w.-]+$/.test(repository)) throw new Error('Invalid GitHub repository');
  const asset = name => {
    if (!statSync(join(directory, name)).size) throw new Error(`Empty update asset: ${name}`);
    const signature = readFileSync(join(directory, `${name}.sig`), 'utf8').trim();
    if (!signature) throw new Error(`Missing signature: ${name}`);
    return { url: `https://github.com/${repository}/releases/download/${tag}/${name}`, signature };
  };
  const mac = asset('screen-buoy-macos.app.tar.gz');
  const windows = asset('screen-buoy-windows-setup.exe');
  return {
    version: tag.slice(1),
    notes: `Release notes: https://github.com/${repository}/releases/tag/${tag}`,
    pub_date: now.toISOString(),
    platforms: { 'darwin-aarch64': mac, 'darwin-x86_64': mac, 'windows-x86_64': windows },
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [, , directory, tag, repository] = process.argv;
  if (!directory || !tag || !repository) throw new Error(`Usage: ${basename(process.argv[1])} DIRECTORY TAG OWNER/REPO`);
  writeFileSync(join(directory, 'latest.json'), `${JSON.stringify(createManifest(directory, tag, repository), null, 2)}\n`);
}

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const version = JSON.parse(read('package.json')).version;
const lock = JSON.parse(read('package-lock.json'));
assert.equal(lock.version, version, 'npm lockfile version');
assert.equal(lock.packages[''].version, version, 'npm root package version');
for (const path of ['src-tauri/Cargo.toml', 'src-tauri/Tauri.toml']) {
  assert.equal(read(path).match(/^version = "([^"]+)"/m)?.[1], version, path);
}
assert.equal(read('src-tauri/Cargo.lock').match(/name = "screen-buoy"\nversion = "([^"]+)"/)?.[1], version, 'Cargo lockfile version');
if (process.env.GITHUB_REF_TYPE === 'tag') {
  assert.equal(process.env.GITHUB_REF_NAME, `v${version}`, 'release tag must match bundle version');
}
console.log(`All package and bundle versions match ${version}`);

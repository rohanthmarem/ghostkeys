// Generate macOS icon assets from the app mark; preserve the Windows ICO.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const icons = join(root, 'src-tauri', 'icons');
const output = mkdtempSync(join(tmpdir(), 'ghostkeys-icons-'));

try {
  execFileSync(process.execPath, [
    join(root, 'node_modules', '@tauri-apps', 'cli', 'tauri.js'),
    'icon',
    join(icons, 'icon.svg'),
    '--output', output,
  ], { cwd: root, stdio: 'inherit' });

  for (const name of ['icon.icns', 'icon.png', '32x32.png', '128x128.png', '128x128@2x.png']) {
    copyFileSync(join(output, name), join(icons, name));
  }
} finally {
  rmSync(output, { recursive: true, force: true });
}

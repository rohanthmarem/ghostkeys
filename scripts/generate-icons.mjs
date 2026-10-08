// Script to generate app icons for Tauri
import sharp from 'sharp';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const ICONS_DIR = 'src-tauri/icons';

// Ensure icons directory exists
mkdirSync(ICONS_DIR, { recursive: true });

// Create a simple ghost icon as SVG
const createGhostSvg = (size) => `
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}">
  <defs>
    <linearGradient id="ghostGrad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:#a78bfa;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#7c3aed;stop-opacity:1" />
    </linearGradient>
  </defs>
  <!-- Background circle -->
  <circle cx="${size/2}" cy="${size/2}" r="${size*0.45}" fill="#1e1b4b"/>
  <!-- Ghost body -->
  <g transform="translate(${size*0.2}, ${size*0.15}) scale(${size/100})">
    <path d="M30 5 C12 5 5 22 5 38 L5 62 L12 55 L20 62 L30 55 L40 62 L48 55 L55 62 L55 38 C55 22 48 5 30 5 Z" fill="url(#ghostGrad)"/>
    <!-- Left eye -->
    <ellipse cx="20" cy="30" rx="6" ry="8" fill="white"/>
    <ellipse cx="22" cy="32" rx="3" ry="4" fill="#1e1b4b"/>
    <!-- Right eye -->
    <ellipse cx="40" cy="30" rx="6" ry="8" fill="white"/>
    <ellipse cx="42" cy="32" rx="3" ry="4" fill="#1e1b4b"/>
  </g>
</svg>
`;

async function generateIcons() {
  const sizes = [32, 128, 256];
  
  console.log('Generating icons...');
  
  // Generate PNG icons
  for (const size of sizes) {
    const svg = Buffer.from(createGhostSvg(size));
    const filename = size === 256 ? '128x128@2x.png' : `${size}x${size}.png`;
    
    await sharp(svg)
      .resize(size, size)
      .png()
      .toFile(join(ICONS_DIR, filename));
    
    console.log(`Created ${filename}`);
  }
  
  // Generate icon.png (used for tray on some systems)
  const svg256 = Buffer.from(createGhostSvg(256));
  await sharp(svg256)
    .resize(256, 256)
    .png()
    .toFile(join(ICONS_DIR, 'icon.png'));
  console.log('Created icon.png');
  
  // Generate ICO for Windows (using 256x256 PNG as base)
  // ICO format: we'll create a simple single-image ICO
  const pngBuffer = await sharp(svg256)
    .resize(256, 256)
    .png()
    .toBuffer();
  
  // Simple ICO file structure for a single 256x256 PNG image
  const icoBuffer = createIcoFromPng(pngBuffer, 256, 256);
  writeFileSync(join(ICONS_DIR, 'icon.ico'), icoBuffer);
  console.log('Created icon.ico');
  
  // ICNS stores PNG images in typed, length-prefixed chunks.
  const chunks = [];
  for (const [type, size] of [['icp4', 16], ['icp5', 32], ['icp6', 64], ['ic07', 128], ['ic08', 256], ['ic09', 512], ['ic10', 1024]]) {
    const png = await sharp(Buffer.from(createGhostSvg(size))).png().toBuffer();
    const header = Buffer.alloc(8);
    header.write(type, 0, 4, 'ascii');
    header.writeUInt32BE(png.length + 8, 4);
    chunks.push(header, png);
  }
  const header = Buffer.alloc(8);
  header.write('icns', 0, 4, 'ascii');
  header.writeUInt32BE(8 + chunks.reduce((total, chunk) => total + chunk.length, 0), 4);
  writeFileSync(join(ICONS_DIR, 'icon.icns'), Buffer.concat([header, ...chunks]));
  console.log('Created icon.icns');
  
  console.log('Done!');
}

// Create a simple ICO file from a PNG buffer
function createIcoFromPng(pngBuffer, width, height) {
  const pngSize = pngBuffer.length;
  
  // ICO header (6 bytes)
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);      // Reserved
  header.writeUInt16LE(1, 2);      // Image type: 1 = ICO
  header.writeUInt16LE(1, 4);      // Number of images
  
  // ICO directory entry (16 bytes)
  const dirEntry = Buffer.alloc(16);
  dirEntry.writeUInt8(width >= 256 ? 0 : width, 0);   // Width (0 means 256)
  dirEntry.writeUInt8(height >= 256 ? 0 : height, 1); // Height (0 means 256)
  dirEntry.writeUInt8(0, 2);       // Color palette
  dirEntry.writeUInt8(0, 3);       // Reserved
  dirEntry.writeUInt16LE(1, 4);    // Color planes
  dirEntry.writeUInt16LE(32, 6);   // Bits per pixel
  dirEntry.writeUInt32LE(pngSize, 8);  // Image size
  dirEntry.writeUInt32LE(22, 12);  // Image offset (6 + 16 = 22)
  
  return Buffer.concat([header, dirEntry, pngBuffer]);
}

generateIcons().catch((error) => { console.error(error); process.exitCode = 1; });

import { existsSync, mkdirSync, renameSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');

const stubs = [
  {
    path: 'node_modules/@covoyage/file2md/dist/transcribe-audio-node.js',
    content: 'export const convertToFlacWithFfmpeg = () => { throw new Error(\'Node.js only\'); };'
  },
  {
    path: 'node_modules/@covoyage/file2md/utils/pdf-pdftotext-node.js',
    content: 'export const extractTextWithPdftotext = () => { throw new Error(\'Node.js only\'); };\nexport const extractPageTextWithPdftotext = () => { throw new Error(\'Node.js only\'); };'
  }
];

// Disable nested pdfjs-dist so esbuild resolves to root version
const nestedPdfjs = join(root, 'node_modules/@covoyage/file2md/node_modules/pdfjs-dist');
const nestedPdfjsBak = nestedPdfjs + '.bak';
if (existsSync(nestedPdfjs) && !existsSync(nestedPdfjsBak)) {
  renameSync(nestedPdfjs, nestedPdfjsBak);
  console.log('Disabled nested pdfjs-dist (moved to pdfjs-dist.bak)');
}
if (!existsSync(nestedPdfjs) && existsSync(nestedPdfjsBak)) {
  console.log('Nested pdfjs-dist already disabled');
}

for (const stub of stubs) {
  const fullPath = join(root, stub.path);
  const dir = dirname(fullPath);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  if (!existsSync(fullPath)) {
    writeFileSync(fullPath, stub.content, 'utf-8');
    console.log(`Created stub: ${stub.path}`);
  }
}

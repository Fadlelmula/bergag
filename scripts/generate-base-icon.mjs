import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 1024;
const CENTER = SIZE / 2;
const TAU = Math.PI * 2;

const crcTable = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buffer) {
  let crc = -1;
  for (const byte of buffer) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeBuffer = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);
  return Buffer.concat([length, typeBuffer, data, crc]);
}

function smoothstep(edge0, edge1, value) {
  const t = Math.min(1, Math.max(0, (value - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

function blend(pixel, color, alpha) {
  const a = Math.min(1, Math.max(0, alpha));
  for (let channel = 0; channel < 3; channel += 1) {
    pixel[channel] = pixel[channel] * (1 - a) + color[channel] * a;
  }
  pixel[3] = Math.max(pixel[3], Math.round(a * 255));
}

function makePng() {
  const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE);
  for (let y = 0; y < SIZE; y += 1) {
    const rowStart = y * (SIZE * 4 + 1);
    raw[rowStart] = 0;
    for (let x = 0; x < SIZE; x += 1) {
      const nx = (x - CENTER) / CENTER;
      const ny = (y - CENTER) / CENTER;
      const radius = Math.hypot(nx, ny);
      const glow = 1 - smoothstep(0.2, 1.15, radius);
      const pixel = [
        10 + Math.round(7 * glow),
        10 + Math.round(5 * glow),
        13 + Math.round(9 * glow),
        255,
      ];

      for (let petal = 0; petal < 6; petal += 1) {
        const theta = (petal / 6) * TAU - Math.PI / 2;
        const cos = Math.cos(theta);
        const sin = Math.sin(theta);
        const u = nx * cos + ny * sin;
        const v = -nx * sin + ny * cos;
        const start = 0.04;
        const end = 0.66;
        if (u < start || u > end) continue;
        const progress = (u - start) / (end - start);
        const curve = 0.055 * Math.sin(progress * Math.PI);
        const width = 0.135 * Math.pow(Math.sin(progress * Math.PI), 0.75);
        const distance = Math.abs(v - curve);
        const alpha = 1 - smoothstep(width * 0.65, width, distance);
        const shade = 0.75 + 0.25 * (1 - progress);
        blend(pixel, [227 * shade, 66 * shade, 87 * shade], alpha * 0.92);
      }

      for (let stamen = 0; stamen < 6; stamen += 1) {
        const theta = (stamen / 6) * TAU - Math.PI / 2 + 0.12;
        const cos = Math.cos(theta);
        const sin = Math.sin(theta);
        const u = nx * cos + ny * sin;
        const v = -nx * sin + ny * cos;
        if (u < 0.02 || u > 0.82) continue;
        const curve = 0.1 * Math.sin((u / 0.82) * Math.PI);
        const distance = Math.abs(v - curve);
        const alpha = 1 - smoothstep(0.004, 0.01, distance);
        blend(pixel, [238, 116, 132], alpha * 0.9);
      }

      // Dark medallion keeps the Bergag monogram legible over the petals while
      // the outer crimson lily and pink stamens remain visible as a halo.
      const centerDistance = Math.hypot(nx, ny);
      blend(pixel, [12, 9, 13], (1 - smoothstep(0.39, 0.44, centerDistance)) * 0.97);
      const rim =
        smoothstep(0.4, 0.425, centerDistance) * (1 - smoothstep(0.44, 0.465, centerDistance));
      blend(pixel, [190, 56, 76], rim * 0.6);

      // Block-style "B" initial for Bergag, drawn as six soft-edged bars.
      const monogram = (x0, y0, x1, y1) => {
        const dx = Math.max(x0 - nx, 0, nx - x1);
        const dy = Math.max(y0 - ny, 0, ny - y1);
        const distance = Math.hypot(dx, dy);
        blend(pixel, [249, 206, 212], (1 - smoothstep(0, 0.014, distance)) * 0.98);
      };
      monogram(-0.27, -0.33, -0.15, 0.33); // spine
      monogram(-0.27, -0.33, 0.07, -0.22); // top bar
      monogram(-0.03, -0.33, 0.09, -0.055); // upper bowl edge
      monogram(-0.27, -0.055, 0.04, 0.055); // middle bar
      monogram(-0.02, 0.055, 0.11, 0.33); // lower bowl edge
      monogram(-0.27, 0.22, 0.1, 0.33); // bottom bar

      const offset = rowStart + 1 + x * 4;
      raw[offset] = pixel[0];
      raw[offset + 1] = pixel[1];
      raw[offset + 2] = pixel[2];
      raw[offset + 3] = pixel[3];
    }
  }

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(SIZE, 0);
  ihdr.writeUInt32BE(SIZE, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const here = dirname(fileURLToPath(import.meta.url));
const target = resolve(here, "..", "assets", "app-icon.png");
mkdirSync(dirname(target), { recursive: true });
writeFileSync(target, makePng());
console.log(`Wrote ${target}`);

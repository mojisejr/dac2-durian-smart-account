// Server-rendering stall probe, without a browser.
//
// Signs in once through Playwright to obtain a session, then drives plain
// HTTP/1.1 requests over persistent sockets, reading every body to its end,
// while other sockets keep the server busy streaming the wasm bundle the way
// a browser would. A page whose response sends its Suspense fallback and then
// never terminates is reported as a stall.
//
// Against the default multi-thread tokio runtime this found 8 stalls in 900
// pages; with one worker thread it found none in 600. It is probabilistic:
// at roughly one stall per hundred pages, 40 rounds miss an existing defect
// about one time in two hundred. Run via scripts/check-stall.sh.
import { chromium } from 'playwright';
import http from 'node:http';

const BASE = process.env.DAC2_BASE_URL ?? 'http://127.0.0.1:3000';
const MAILPIT = process.env.DAC2_MAILPIT_URL ?? 'http://127.0.0.1:8025';
const email = 'stall-probe@dac2.local';
const password = 'CorrectHorse123!';

const browser = await chromium.launch({ channel: 'chrome' });
const context = await browser.newContext();
const page = await context.newPage();
await page.goto(`${BASE}/register`);
await page.fill('input[name="email"]', email);
await page.fill('input[name="password"]', password);
await page.fill('input[name="password_confirm"]', password);
await page.click('button[type="submit"]');
await page.waitForTimeout(1500);
const search = await (await fetch(`${MAILPIT}/api/v1/search?query=to:${email}`)).json();
const id = search.messages?.[0]?.ID;
if (id) {
  const message = await (await fetch(`${MAILPIT}/api/v1/message/${id}`)).json();
  const link = (message.Text || '').match(/https?:\/\/[^\s]*verify-email\?token=[A-Za-z0-9]+/)?.[0];
  if (link) await page.goto(link);
}
await page.goto(`${BASE}/login`);
await page.fill('input[name="email"]', email);
await page.fill('input[name="password"]', password);
await page.click('button[type="submit"]');
await page.waitForTimeout(1200);
await page.goto(`${BASE}/plans/new`);
await page.fill('input[name="season_year"]', '2569');
await page.fill('input[name="name"]', 'stall probe');
await page.click('button:has-text("สร้างฤดูกาล")');
await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
const planId = page.url().match(/\/plans\/(\d+)\//)[1];
await page.goto(`${BASE}/plans/${planId}`);
const modeButton = page.locator('button:has-text("เปลี่ยนเป็นแผนละเอียด")');
if (await modeButton.count()) { await modeButton.click(); await page.waitForTimeout(800); }
const cookie = (await context.cookies()).map((c) => `${c.name}=${c.value}`).join('; ');
await browser.close();

const routes = ['', '/dashboard', '/analysis', '/market', '/production', '/expenses',
  '/variable-costs', '/fixed-costs', '/health', '/targets', '/assets', '/close', '/comparison']
  .map((r) => `/plans/${planId}${r}`).concat(['/plans', '/history']);

const agent = new http.Agent({ keepAlive: true, maxSockets: 3 });
const ROUNDS = Number(process.env.ROUNDS ?? 40);
let stalls = 0;
const get = (path) => new Promise((resolve) => {
  const started = Date.now();
  const req = http.get(BASE + path, { agent, headers: { cookie } }, (res) => {
    let bytes = 0;
    let headersAt = Date.now() - started;
    let last = '';
    const timer = setTimeout(() => {
      stalls += 1;
      console.log(`STALL ${path}: status ${res.statusCode} headers at ${headersAt}ms, ${bytes} bytes then silence 10s; tail=${JSON.stringify(last.slice(-80))}; te=${res.headers['transfer-encoding']} cl=${res.headers['content-length']}`);
      req.destroy();
      resolve('stall');
    }, 10000);
    res.on('data', (chunk) => { bytes += chunk.length; last = chunk.toString('utf8'); });
    res.on('end', () => { clearTimeout(timer); resolve({ status: res.statusCode, bytes, ms: Date.now() - started }); });
    res.on('error', () => { clearTimeout(timer); resolve('error'); });
  });
  req.on('error', (e) => { console.log(`ERROR ${path}: ${e.message}`); resolve('error'); });
  req.setTimeout(10000, () => { stalls += 1; console.log(`STALL ${path}: no headers in 10s`); req.destroy(); resolve('stall'); });
});
const LOAD = Number(process.env.LOAD ?? 3);
const loadAgent = new http.Agent({ keepAlive: true, maxSockets: LOAD });
const wasm = () => new Promise((resolve) => {
  http.get(BASE + '/pkg/dac2.wasm', { agent: loadAgent }, (res) => { res.resume(); res.on('end', resolve); res.on('error', resolve); }).on('error', resolve);
});
for (let round = 0; round < ROUNDS; round += 1) {
  for (const path of routes) {
    // Chrome fetches the 19.5 MB wasm and other assets alongside every page;
    // keep the server busy the same way while the page renders.
    const busy = Promise.all(Array.from({ length: LOAD }, wasm));
    const r = await get(path);
    await busy;
    if (r === 'stall' && stalls >= 3) { round = ROUNDS; break; }
  }
  if (round % 10 === 9) console.log(`round ${round + 1}: ${stalls} stalls so far`);
}
console.log(`done: ${routes.length} routes x up to ${ROUNDS} rounds, ${stalls} stalls`);
agent.destroy(); loadAgent.destroy();
process.exit(stalls === 0 ? 0 : 1);

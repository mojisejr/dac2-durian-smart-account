// Layout proof for the rules DESIGN.md calls non-negotiable.
//
// The Rust suites render components to an HTML string, which can prove what a
// screen says and never what it does on a phone. This drives a real browser at
// the widths the owner's readers actually hold, and asserts three properties
// that a string cannot carry:
//
//   1. No page is wider than the device. The layout viewport may not be pushed
//      out by content, which is how one table once widened every screen to 705px.
//   2. No interactive label is clipped by its own box.
//   3. Every activation target meets the 48px minimum of DESIGN.md rule 1, and
//      nothing that must be tapped is covered by the sticky bars.
//
// It fails loudly rather than skipping, because a check that can quietly opt out
// of a non-negotiable rule is how the rule got broken in the first place.

import { chromium } from 'playwright';
import { execSync } from 'node:child_process';

const BASE = process.env.DAC2_BASE_URL ?? 'http://127.0.0.1:3000';
const MAILPIT = process.env.DAC2_MAILPIT_URL ?? 'http://127.0.0.1:8025';

// The narrowest phone still in use, the two common sizes, and a large Android.
const WIDTHS = [
  { name: '320', width: 320, height: 568 },
  { name: '360', width: 360, height: 740 },
  { name: '393', width: 393, height: 852 },
  { name: '412', width: 412, height: 915 },
];

const failures = [];
const fail = (where, message) => failures.push(`${where}: ${message}`);

async function submitAndReload(page, button, label) {
  const pending = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await button.click();
  const response = await pending;
  if (!response.ok()) {
    throw new Error(`${label} returned HTTP ${response.status()}`);
  }
  await page.waitForTimeout(300);
  const error = (await page.locator('.bad-message').allTextContents()).join(' ').trim();
  if (error) throw new Error(`${label} failed: ${error}`);
  await reloadOrDiagnose(page);
}

// Submits an action and leaves the page where it is. The caller then waits for
// the change to appear, and `assertNoReload` proves the page showed it by
// reading its data again rather than by a navigation, which is the defect the
// owner met on the assets page: the action saved, the screen stayed stale.
async function submitInPlace(page, button, label) {
  await page.evaluate(() => { window.__dac2InPlace = true; });
  const pending = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await button.click();
  const response = await pending;
  if (!response.ok()) {
    throw new Error(`${label} returned HTTP ${response.status()}`);
  }
  await page.waitForTimeout(300);
  const error = (await page.locator('.bad-message').allTextContents()).join(' ').trim();
  if (error) throw new Error(`${label} failed: ${error}`);
}

async function assertNoReload(page, label) {
  const kept = await page.evaluate(() => window.__dac2InPlace === true);
  if (!kept) throw new Error(`${label}: the page reloaded instead of updating in place`);
}

async function inspect(page, intendedWidth) {
  return page.evaluate((intendedWidth) => {
    const inScroller = (el) => {
      for (let node = el.parentElement; node; node = node.parentElement) {
        const overflow = getComputedStyle(node).overflowX;
        if (overflow === 'auto' || overflow === 'scroll' || overflow === 'hidden') return true;
      }
      return false;
    };
    const visible = (el) => {
      const style = getComputedStyle(el);
      if (style.visibility === 'hidden' || style.display === 'none') return false;
      if (el.closest('details:not([open])') && !el.matches('summary, details')) return false;
      const rect = el.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0;
    };
    const name = (el) => {
      const classes = typeof el.className === 'string' ? el.className.trim().split(/\s+/).filter(Boolean) : [];
      return el.tagName.toLowerCase() + (classes.length ? '.' + classes.join('.') : '');
    };

    const overflowing = [];
    const clipped = [];
    for (const el of document.querySelectorAll('*')) {
      if (!visible(el)) continue;
      const rect = el.getBoundingClientRect();
      // Content inside a horizontal scroller is meant to extend past the screen.
      if (!inScroller(el) && rect.right > intendedWidth + 1) {
        overflowing.push({ el: name(el), over: Math.round(rect.right - intendedWidth), text: (el.textContent || '').trim().slice(0, 40) });
      }
      if (el.children.length === 0 && el.scrollWidth > el.clientWidth + 1) {
        const style = getComputedStyle(el);
        // Scrolling and an ellipsis are deliberate: both tell the reader there
        // is more. Silent clipping is the defect.
        const deliberate =
          style.overflowX === 'auto' ||
          style.overflowX === 'scroll' ||
          style.textOverflow === 'ellipsis';
        if (!deliberate) {
          clipped.push({
            el: name(el),
            field: el instanceof HTMLInputElement ? `${el.type}:${el.name}=${el.value}`.slice(0, 60) : '',
            text: (el.textContent || '').trim().slice(0, 40),
          });
        }
      }
    }

    // DESIGN.md rule 2: every text colour clears 6:1 against the surface behind
    // it. It is as measurable as the 48px rule and was never measured, which is
    // how a sheet came to render its text at 1.04:1 and still pass.
    const parse = (value) => {
      const parts = value.match(/[\d.]+/g);
      if (!parts) return null;
      const channels = parts.slice(0, 3).map(Number);
      // Chromium serializes color-mix() as color(srgb 0..1 0..1 0..1).
      // Treating those fractions as legacy rgb(0..255) makes a pale surface
      // look black and reports a false contrast failure.
      return value.startsWith('color(srgb')
        ? channels.map((channel) => channel * 255)
        : channels;
    };
    const alpha = (value) => {
      const parts = value.match(/[\d.]+/g);
      return parts && parts.length > 3 ? Number(parts[3]) : 1;
    };
    const luminance = ([r, g, b]) => {
      const channel = (c) => {
        const v = c / 255;
        return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
      };
      return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    };
    const behind = (el) => {
      for (let node = el; node; node = node.parentElement) {
        const bg = getComputedStyle(node).backgroundColor;
        if (alpha(bg) > 0.95) return parse(bg);
      }
      return [255, 255, 255];
    };
    const lowContrast = [];
    for (const el of document.querySelectorAll('p, h1, h2, h3, span, strong, small, a, button, summary, label, th, td, caption, legend')) {
      if (!visible(el)) continue;
      const text = Array.from(el.childNodes)
        .filter((node) => node.nodeType === 3)
        .map((node) => node.textContent.trim())
        .join('');
      if (!text) continue;
      const style = getComputedStyle(el);
      const front = parse(style.color);
      const back = behind(el);
      if (!front || !back) continue;
      const lighter = Math.max(luminance(front), luminance(back));
      const darker = Math.min(luminance(front), luminance(back));
      const ratio = (lighter + 0.05) / (darker + 0.05);
      if (ratio < 6) {
        lowContrast.push({ el: name(el), ratio: ratio.toFixed(2), text: text.slice(0, 34) });
      }
    }

    const scrollTop = window.scrollY;
    const small = [];
    const covered = [];
    for (const el of document.querySelectorAll('a[href], button, summary, [role="tab"], input[type="range"]')) {
      if (!visible(el)) continue;
      // A control wrapped in a label is activated by the whole label.
      const target = el.closest('label') ?? el;
      const rect = target.getBoundingClientRect();
      if (rect.height < 47.5 || rect.width < 47.5) {
        small.push({ el: name(el), w: Math.round(rect.width), h: Math.round(rect.height), text: (el.textContent || '').trim().slice(0, 30) });
      }
      // Nothing that must be tapped may sit under the sticky bars. A control
      // under the bar right now is fine if scrolling brings it clear, so the
      // test is whether it is still covered once scrolled to the middle of the
      // screen — the position a thumb would actually bring it to. While a sheet
      // is up the whole screen is blocked on purpose, so the question does not
      // apply and is not asked.
      if (document.querySelector('details[open] > .sheet')) continue;
      el.scrollIntoView({ block: 'center', behavior: 'instant' });
      const placed = target.getBoundingClientRect();
      const x = Math.min(Math.max(placed.left + placed.width / 2, 1), intendedWidth - 1);
      const y = placed.top + placed.height / 2;
      if (y > 0 && y < window.innerHeight) {
        const hit = document.elementFromPoint(x, y);
        // An open explanation is a transient overlay the reader dismisses.
        // Persistent chrome is not, and that is the only kind that fails here.
        const chrome = hit?.closest('.bottom-nav, .live-total');
        const transient = hit?.closest('details[open]');
        if (hit && chrome && !transient && !el.contains(hit) && !hit.contains(el)) {
          covered.push({ el: name(el), by: name(chrome), text: (el.textContent || '').trim().slice(0, 30) });
        }
      }
    }

    window.scrollTo(0, scrollTop);
    // An open explanation is the state that failed twice: first by overflowing
    // the screen, then by showing 49% of its text with nothing to say the rest
    // existed. It must lie wholly on screen and carry a real close control.
    const sheets = [];
    for (const sheet of document.querySelectorAll('details[open] > .sheet')) {
      const rect = sheet.getBoundingClientRect();
      const close = sheet.querySelector('.sheet-close');
      const closeRect = close?.getBoundingClientRect();
      const body = sheet.querySelector('.sheet-body');
      sheets.push({
        label: sheet.querySelector('.sheet-title')?.textContent?.trim().slice(0, 30) ?? '?',
        outside:
          rect.left < -1 ||
          rect.right > intendedWidth + 1 ||
          rect.top < -1 ||
          rect.bottom > window.innerHeight + 1,
        closeWidth: closeRect ? Math.round(closeRect.width) : 0,
        closeHeight: closeRect ? Math.round(closeRect.height) : 0,
        bodyScrolls: body ? getComputedStyle(body).overflowY === 'auto' : false,
        bodyVisible: body ? Math.round(body.clientHeight) : 0,
        bodyNeeded: body ? Math.round(body.scrollHeight) : 0,
      });
    }

    return {
      lowContrast,
      sheets,
      laidOutAt: window.innerWidth,
      documentWidth: document.documentElement.scrollWidth,
      overflowing,
      clipped,
      small,
      covered,
    };
  }, intendedWidth);
}

function assess(where, report, intendedWidth) {
  if (report.laidOutAt > intendedWidth) {
    fail(where, `the layout viewport was pushed from ${intendedWidth}px to ${report.laidOutAt}px by content`);
  }
  if (report.documentWidth > intendedWidth + 1) {
    fail(where, `the document is ${report.documentWidth}px wide on a ${intendedWidth}px screen`);
  }
  for (const item of report.overflowing.slice(0, 4)) {
    fail(where, `${item.el} runs ${item.over}px past the right edge — "${item.text}"`);
  }
  for (const item of report.clipped.slice(0, 4)) {
    const detail = item.field || item.text;
    fail(where, `${item.el} clips its own text — "${detail}"`);
  }
  for (const item of report.small.slice(0, 4)) {
    fail(where, `${item.el} is ${item.w}x${item.h}, under the 48px rule — "${item.text}"`);
  }
  for (const item of report.covered.slice(0, 4)) {
    fail(where, `${item.el} is covered by ${item.by} and cannot be tapped — "${item.text}"`);
  }
  for (const item of report.lowContrast.slice(0, 4)) {
    fail(where, `${item.el} reads at ${item.ratio}:1, under the 6:1 rule — "${item.text}"`);
  }
  for (const sheet of report.sheets) {
    if (sheet.outside) {
      fail(where, `the "${sheet.label}" explanation extends past the screen`);
    }
    if (sheet.closeHeight < 47.5 || sheet.closeWidth < 47.5) {
      fail(where, `the "${sheet.label}" explanation closes with a ${sheet.closeWidth}x${sheet.closeHeight} control, under the 48px rule`);
    }
    // Text longer than the room it has must be scrollable, or the reader is
    // shown part of an answer with nothing to say the rest exists.
    if (sheet.bodyNeeded > sheet.bodyVisible + 1 && !sheet.bodyScrolls) {
      fail(where, `the "${sheet.label}" explanation needs ${sheet.bodyNeeded}px, shows ${sheet.bodyVisible}px, and cannot be scrolled`);
    }
  }
}


// Guided fields bind their value and checked state as properties, so the
// server-rendered markup carries no value until hydration runs. Wait for the
// hydrated value instead of reading whatever the pre-hydration DOM holds.
async function waitForHydratedValue(page, selector, accept, what) {
  await page
    .waitForFunction(
      ([selector, acceptSource]) => {
        const accept = new Function('value', `return (${acceptSource})(value);`);
        return [...document.querySelectorAll(selector)].some((element) =>
          accept(element.type === 'radio' || element.type === 'checkbox' ? element.checked : element.value),
        );
      },
      [selector, accept.toString()],
      { timeout: 10000 },
    )
    .catch(async () => {
      const seen = await page
        .locator(selector)
        .evaluateAll((elements) => elements.map((element) => (element.type === 'radio' || element.type === 'checkbox' ? element.checked : element.value)))
        .catch(() => '(missing)');
      const messages = await page
        .locator('.form-message, .validation-summary, .field-error')
        .allTextContents()
        .then((texts) => texts.map((text) => text.trim()).filter(Boolean))
        .catch(() => []);
      throw new Error(`${what} (saw ${JSON.stringify(seen)} at ${page.url()}; page says ${JSON.stringify(messages)})`);
    });
}

// A navigation that hangs is only useful if it says where it hung. On a
// timeout, ask the server for the same page without the browser and list the
// requests Chrome still had open, so the failure names the side that stalled.
async function gotoOrDiagnose(page, url, cookies) {
  await navigateOrDiagnose(page, url, cookies, () =>
    page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 }));
}

async function reloadOrDiagnose(page) {
  await navigateOrDiagnose(page, page.url(), null, () =>
    page.reload({ waitUntil: 'domcontentloaded' }));
}

// `cookies` may be null: the diagnosis then reads them from the page's context.
async function navigateOrDiagnose(page, url, cookies, navigate) {
  const pending = new Map();
  const onRequest = (request) => pending.set(request, Date.now());
  const onDone = (request) => pending.delete(request);
  page.on('request', onRequest);
  page.on('requestfinished', onDone);
  page.on('requestfailed', onDone);
  try {
    await navigate();
  } catch (error) {
    const jar = cookies ?? (await page.context().cookies());
    const cookie = jar.map((c) => `${c.name}=${c.value}`).join('; ');
    const started = Date.now();
    let direct;
    try {
      const response = await fetch(url, {
        headers: { cookie },
        redirect: 'manual',
        signal: AbortSignal.timeout(10000),
      });
      direct = `server answered HTTP ${response.status} in ${Date.now() - started}ms`;
    } catch (fetchError) {
      direct = `server did not answer within 10s (${fetchError.message})`;
    }
    // Chrome's own timing says whether a stalled request ever reached the
    // wire: requestStart stays -1 while it waits in the connection queue.
    // Server-side view of the same moment: which sockets the application
    // holds on its port and in what TCP state, and what every database
    // session is doing, so a stall names the layer that stopped answering.
    const shell = (command) => {
      try {
        return execSync(command, { encoding: 'utf8', timeout: 10000 }).trim() || '(none)';
      } catch (shellError) {
        return `(failed: ${shellError.message.split('\n')[0]})`;
      }
    };
    const port = new URL(BASE).port || '80';
    const sockets = shell(`lsof -nP -iTCP:${port} 2>/dev/null | tail -n +2`);
    const dbSessions = shell(
      "docker compose exec -T database psql -U postgres -d dac2 -At -F ' | ' -c " +
      "\"SELECT pid, state, wait_event_type, wait_event, now() - query_start AS age, left(regexp_replace(query, '\\s+', ' ', 'g'), 120) FROM pg_stat_activity WHERE datname = 'dac2' AND pid <> pg_backend_pid() ORDER BY query_start\"",
    );
    const open = [...pending.entries()]
      .map(([request, at]) => {
        const timing = request.timing();
        const wire = timing.requestStart >= 0
          ? `sent at +${Math.round(timing.requestStart)}ms, connect ${Math.round(timing.connectStart)}..${Math.round(timing.connectEnd)}ms`
          : `never sent (connectStart ${Math.round(timing.connectStart)}ms)`;
        return `${request.method()} ${request.url()} open ${Date.now() - at}ms, ${wire}`;
      })
      .join('; ');
    throw new Error(
      `${error.message}\n  ${direct}\n  browser requests still open: ${open || 'none'}` +
      `\n  application sockets on :${port}:\n${sockets.replace(/^/gm, '    ')}` +
      `\n  database sessions:\n${dbSessions.replace(/^/gm, '    ')}`,
    );
  } finally {
    page.off('request', onRequest);
    page.off('requestfinished', onDone);
    page.off('requestfailed', onDone);
  }
}

async function signIn(browser) {
  const email = 'responsive-proof@dac2.local';
  const password = 'CorrectHorse123!';
  const context = await browser.newContext({ viewport: { width: 412, height: 915 } });
  const page = await context.newPage();

  await navigateOrDiagnose(page, `${BASE}/register`, null, () => page.goto(`${BASE}/register`));
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForTimeout(1500);

  const search = await (await fetch(`${MAILPIT}/api/v1/search?query=to:${email}`)).json();
  const id = search.messages?.[0]?.ID;
  if (!id) throw new Error('registration sent no mail to Mailpit');
  const message = await (await fetch(`${MAILPIT}/api/v1/message/${id}`)).json();
  const link = (message.Text || '').match(/https?:\/\/[^\s]*verify-email\?token=[A-Za-z0-9]+/)?.[0];
  if (!link) throw new Error('the verification mail carries no link');
  await navigateOrDiagnose(page, link, null, () => page.goto(link));

  await navigateOrDiagnose(page, `${BASE}/login`, null, () => page.goto(`${BASE}/login`));
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForTimeout(1200);

  // Before any season exists, the season list and the history are empty
  // states; measure them at every width now, because the matrix at the end
  // runs with seasons in place.
  {
    const cookiesNow = await context.cookies();
    for (const size of WIDTHS) {
      const emptyContext = await browser.newContext({
        viewport: { width: size.width, height: size.height },
        isMobile: true,
        hasTouch: true,
        deviceScaleFactor: 2,
      });
      await emptyContext.addCookies(cookiesNow);
      for (const [name, url] of [['plans-empty', `${BASE}/plans`], ['history-empty', `${BASE}/history`]]) {
        const emptyPage = await emptyContext.newPage();
        try {
          await gotoOrDiagnose(emptyPage, url, cookiesNow);
          await emptyPage.waitForTimeout(250);
          assess(`${size.name}px ${name}`, await inspect(emptyPage, size.width), size.width);
        } finally {
          await emptyPage.close();
        }
      }
      await emptyContext.close();
    }
  }

  // The workbook sample is a browser-only sandbox. Editing and leaving it must
  // not create a season row.
  await navigateOrDiagnose(page, `${BASE}/demo`, null, () => page.goto(`${BASE}/demo`));
  await page.getByLabel('ราคาเกรดแรก').fill('120');
  await page.click('button:has-text("คืนค่าตัวอย่าง")');
  await navigateOrDiagnose(page, `${BASE}/plans`, null, () => page.goto(`${BASE}/plans`));
  if (await page.locator('.plan-list-item').count()) {
    throw new Error('the browser-only demonstration created a stored season');
  }

  await navigateOrDiagnose(page, `${BASE}/plans/new`, null, () => page.goto(`${BASE}/plans/new`));
  await page.fill('input[name="season_year"]', '2569');
  await page.fill('input[name="name"]', 'สวนรวมสำหรับ responsive proof');
  await page.fill('textarea[name="note"]', 'ข้อมูลทดสอบที่สคริปต์ลบพร้อมบัญชี');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
  const planId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!planId) throw new Error('the quick season was not created');

  await page.fill('input[name="value"]', '20000');
  await page.click('button:has-text("ถัดไป")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/price$`));
  if (await page.locator('input[name="value"]').inputValue() !== '') {
    throw new Error('quick production leaked into the price field after client navigation');
  }

  // Leaving after one answer and returning through the hub must resume at the
  // first unanswered question rather than restarting or inventing a result.
  await navigateOrDiagnose(page, `${BASE}/plans/${planId}`, null, () => page.goto(`${BASE}/plans/${planId}`));
  await page.click('a:has-text("ทำประมาณการต่อ")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/price$`));
  await page.fill('input[name="value"]', '80');
  await page.click('button:has-text("ถัดไป")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/cost$`));
  if (await page.locator('input[name="value"]').inputValue() !== '') {
    throw new Error('quick price leaked into the cost field after client navigation');
  }
  await page.goBack({ waitUntil: 'domcontentloaded' });
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/price$`));
  if (await page.locator('input[name="value"]').inputValue() !== '80') {
    throw new Error('browser back lost the persisted quick price');
  }
  await page.click('button:has-text("ถัดไป")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/cost$`));
  await reloadOrDiagnose(page);
  await page.waitForTimeout(400);
  await page.fill('input[name="value"]', '900000');
  await page.click('button:has-text("ดูผลประมาณการ")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/result$`));

  // Save a reviewable actual draft. Leaving review must keep the season open
  // and the draft available when the owner returns.
  await navigateOrDiagnose(page, `${BASE}/plans/${planId}/close`, null, () => page.goto(`${BASE}/plans/${planId}/close`));
  await page.fill('input[name="sellable_yield_kg"]', '18000');
  await page.fill('input[name="revenue"]', '1530000');
  await page.fill('input[name="total_cost"]', '990000');
  await page.fill('textarea[name="note"]', 'ผลผลิตน้อยกว่าคาด');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${planId}/close/review$`));
  await page.click('a:has-text("ยกเลิก · ยังไม่ปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${planId}$`));
  await page
    .locator('a:has-text("บันทึกผลจริงและปิดฤดูกาล")')
    .waitFor({ state: 'visible', timeout: 10000 })
    .catch(() => {
      throw new Error('leaving actual review closed the season unexpectedly');
    });
  await navigateOrDiagnose(page, `${BASE}/plans/${planId}/close/review`, null, () => page.goto(`${BASE}/plans/${planId}/close/review`));

  // Keep a separate detailed-mode season in the same account so this proof
  // covers the new quick surfaces without dropping the pre-existing detailed
  // dashboard, analysis, and long-form input regression.
  await navigateOrDiagnose(page, `${BASE}/plans/new`, null, () => page.goto(`${BASE}/plans/new`));
  await page.fill('input[name="season_year"]', '2570');
  await page.fill('input[name="name"]', 'สวนทดสอบละเอียด');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/).catch(async () => {
    const message = (await page.locator('.form-message').allTextContents()).join(' ').trim();
    throw new Error(`detailed season stayed at ${page.url()}: ${message || 'no visible server message'}`);
  });
  const detailedPlanId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!detailedPlanId) throw new Error('the detailed proof season was not created');
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  const modeResponse = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await page.click('button:has-text("เปลี่ยนเป็นแผนละเอียด")');
  const modeResult = await modeResponse;
  if (!modeResult.ok()) {
    throw new Error(`switching to detailed mode returned HTTP ${modeResult.status()}`);
  }
  await reloadOrDiagnose(page);
  await page
    .getByRole('heading', { name: 'แผนละเอียด', exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/quick/production`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/quick/production`));
  await page
    .getByRole('heading', { name: 'แผนละเอียดกำลังใช้งาน', exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${planId}/production`, null, () => page.goto(`${BASE}/plans/${planId}/production`));
  await page
    .getByRole('heading', { name: 'ประมาณการเร็วกำลังใช้งาน', exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });

  // Batch 2: the production page offers two ways to answer each question.
  // Switching branches must keep the other branch's typed facts, refresh and
  // back must keep the saved selection, and the unselected branch's figure is
  // quoted beside the entry rather than applied.
  const production = `${BASE}/plans/${detailedPlanId}/production`;
  await navigateOrDiagnose(page, production, null, () => page.goto(production));
  await page.locator('#production-trees').waitFor({ state: 'visible', timeout: 10000 });
  // The selection is bound as a property, so it appears only once hydration
  // has run; wait for it rather than reading the server markup's blank.
  await waitForHydratedValue(page, 'input[name="yield-source"][value="derived"]', (v) => v === true, 'a new detailed plan does not start on the derived yield branch');
  await page.fill('#production-trees', '200');
  await page.fill('#production-fruits-per-tree', '35');
  await page.fill('#production-fruit-weight', '3');
  await page.fill('#production-loss', '5');
  await page.getByText('คำนวณได้ประมาณ 19,950.00 กก. หลังหักส่วนที่เสีย').waitFor({ timeout: 5000 });
  await page.check('input[name="yield-source"][value="direct"]');
  await page.locator('#production-sellable-yield').waitFor({ state: 'visible', timeout: 5000 });
  if (await page.locator('#production-trees').count()) {
    throw new Error('the unselected derived fields are still asked on the direct branch');
  }
  await page.getByText('จากข้อมูลต้นทุเรียนที่กรอกไว้ คำนวณได้ประมาณ 19,950.00 กก.').waitFor({ timeout: 5000 });
  await page.fill('#production-sellable-yield', '18000');
  await page.check('input[name="yield-source"][value="derived"]');
  if ((await page.locator('#production-trees').inputValue()) !== '200') {
    throw new Error('switching yield branches lost the derived facts');
  }
  await page.check('input[name="yield-source"][value="direct"]');
  if (!['18000', '18,000'].includes(await page.locator('#production-sellable-yield').inputValue())) {
    throw new Error('switching yield branches lost the direct figure');
  }
  if (await page.isChecked('input[name="grade-entry"][value="kilograms"]')) {
    throw new Error('grade entry started in kilograms without an explicit choice');
  }
  await page.check('input[name="price-source"][value="average"]');
  await page.fill('#production-average-price', '79');
  if (await page.locator('input[name="grade-entry"]').count()) {
    throw new Error('grade entry controls are shown while one average price is selected');
  }
  const saveResponse = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await page.click('button:has-text("บันทึกส่วนนี้")');
  if (!(await saveResponse).ok()) {
    throw new Error('saving the production branches returned a failing HTTP status');
  }
  await page.locator('.form-message:not(:empty)').first().waitFor({ timeout: 10000 });
  await reloadOrDiagnose(page);
  await waitForHydratedValue(page, 'input[name="yield-source"][value="direct"]', (v) => v === true, 'refresh lost the saved direct yield branch');
  await waitForHydratedValue(page, '#production-sellable-yield', (v) => v === '18000', 'refresh lost the saved direct sellable kilograms');
  await waitForHydratedValue(page, '#production-average-price', (v) => v === '79', 'refresh lost the saved average price');
  await page.check('input[name="yield-source"][value="derived"]');
  if ((await page.locator('#production-trees').inputValue()) !== '200') {
    throw new Error('the unselected derived facts were not persisted with the section');
  }

  // Interrupted resume: leave for the market page, answer the buyer question,
  // and come back through browser history without losing either page.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/market`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/market`));
  await page.locator('#market-buyer-committed-kg').waitFor({ state: 'visible', timeout: 10000 });
  await page.fill('#market-buyer-committed-kg', '25000');
  const marketSave = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await page.click('button:has-text("บันทึกส่วนนี้")');
  const marketResponse = await marketSave;
  if (!marketResponse.ok()) {
    throw new Error(`saving the market section returned HTTP ${marketResponse.status()} from ${marketResponse.url()}`);
  }
  await page.locator('.form-message:not(:empty)').first().waitFor({ timeout: 10000 });
  await navigateOrDiagnose(page, production, null, () => page.goto(production));
  await page.locator('#production-sellable-yield').waitFor({ state: 'visible', timeout: 10000 });
  await page.goBack({ waitUntil: 'domcontentloaded' });
  // A restored history entry keeps the blur-formatted text; a re-rendered
  // one shows the stored figure. Either is the same saved answer. Owner pages
  // are served no-store, so back never replays the page from before the save.
  await waitForHydratedValue(page, '#market-buyer-committed-kg', (v) => v === '25000' || v === '25,000', 'browser back lost the saved buyer quantity');
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.getByText('พร้อมเทียบยอดผู้ซื้อกับผลผลิต').waitFor({ timeout: 10000 });

  // Grade entry in kilograms converts visibly against the saved total.
  await navigateOrDiagnose(page, production, null, () => page.goto(production));
  await waitForHydratedValue(page, '#production-sellable-yield', (v) => v === '18000', 'the saved sellable kilograms did not hydrate before grade entry');
  await page.check('input[name="price-source"][value="by_grade"]');
  await page.click('button:has-text("+ เพิ่มเกรด")');
  await page.check('input[name="grade-entry"][value="kilograms"]');
  await page.getByLabel('เกรดนี้กี่กิโล').fill('9000');
  await page.getByText('≈ 50.00% ของกิโลที่คาดว่าจะขายได้').waitFor({ timeout: 5000 });
  await page.check('input[name="grade-entry"][value="percent"]');
  if ((await page.getByLabel('เกรดนี้กี่เปอร์เซ็นต์ของทั้งหมด').inputValue()) !== '50') {
    throw new Error('switching grade entry back to percent did not show the converted share');
  }
  await page.check('input[name="price-source"][value="average"]');
  await page.getByText('เกรดที่เคยกรอกไว้ยังเก็บอยู่').waitFor({ timeout: 5000 });

  // Batch 3: a remembered expense is captured before it is classified, the
  // hub names it as waiting, leaving and returning keeps it, classifying moves
  // it, and confirming a section empty changes the result from unavailable to
  // a figure.
  const expenses = `${BASE}/plans/${detailedPlanId}/expenses`;
  await navigateOrDiagnose(page, expenses, null, () => page.goto(expenses));
  await page.click('button:has-text("+ จดค่าใช้จ่ายที่จำได้")');
  await page.getByLabel('ค่าอะไร').first().fill('จ่ายคนขับรถเดือนสาม');
  await page.getByLabel('เท่าไร').first().fill('50000');
  await page.getByText('ยังมี 1 รายการที่ยังไม่ได้บอกว่าเป็นแบบไหน จึงยังไม่ถูกนับ').waitFor({ timeout: 5000 });
  const captureSave = page.waitForResponse((r) => r.request().method() === 'POST', { timeout: 10000 });
  await page.click('button:has-text("บันทึกส่วนนี้")');
  const captureResponse = await captureSave;
  if (!captureResponse.ok()) throw new Error(`saving a captured expense returned HTTP ${captureResponse.status()} from ${captureResponse.url()}`);
  await page.locator('.form-message:not(:empty)').first().waitFor({ timeout: 10000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.getByText('มี 1 รายการยังไม่ได้บอกว่าเป็นแบบไหน').waitFor({ timeout: 10000 });
  await page.getByRole('heading', { name: 'บอกว่าค่าใช้จ่ายที่จดไว้เป็นแบบไหน', exact: true }).waitFor({ timeout: 10000 });
  await navigateOrDiagnose(page, expenses, null, () => page.goto(expenses));
  await waitForHydratedValue(page, '.repeat-row input[type="text"]', (v) => v === 'จ่ายคนขับรถเดือนสาม', 'the captured expense did not survive leaving and returning');
  await page.click('button:has-text("บอกว่าเป็นแบบไหน")');
  await page.check('input[name="classify-grows"][value="yes"]');
  await page.locator('.classify-flow select').selectOption('transport');
  await page.click('button:has-text("ย้ายไปส่วนที่ถูก")');
  if (await page.locator('button:has-text("บอกว่าเป็นแบบไหน")').count()) {
    throw new Error('classifying did not remove the item from the waiting list');
  }
  const classifySave = page.waitForResponse((r) => r.request().method() === 'POST', { timeout: 10000 });
  await page.click('button:has-text("บันทึกส่วนนี้")');
  if (!(await classifySave).ok()) throw new Error('saving a classified expense returned a failing HTTP status');
  await page.locator('.form-message:not(:empty)').first().waitFor({ timeout: 10000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/variable-costs`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/variable-costs`));
  await page.getByLabel('ยอดรวมทั้งฤดู').waitFor({ state: 'visible', timeout: 10000 });
  await waitForHydratedValue(page, '.repeat-row input[inputmode="decimal"]', (v) => v === '50000' || v === '50,000', 'the classified expense lost its amount on the way to variable costs');
  if (await page.locator('input[name="variable-cost-state"]').count()) {
    throw new Error('the unknown/confirmed-none question is still asked while rows exist');
  }

  // Keystrokes into a repeat-row number field must all land. A list that
  // re-rendered on every change recreated the focused input after the first
  // character; the owner saw one digit and then nothing.
  await page.click('button:has-text("+ เพิ่มค่าใช้จ่าย")');
  const quantity = page.locator('.repeat-row input[inputmode="decimal"]').last();
  await quantity.click();
  await page.keyboard.type('12345', { delay: 40 });
  if ((await quantity.inputValue()) !== '12345') {
    throw new Error(`a repeat-row number field dropped keystrokes (saw ${JSON.stringify(await quantity.inputValue())})`);
  }
  await page.click('.repeat-row:last-of-type button:has-text("ลบรายการ")');

  // With variable costs entered and fixed costs unknown the profit is still
  // unavailable; confirming the fixed section empty makes it a figure.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/fixed-costs`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/fixed-costs`));
  await page.locator('input[name="fixed-cost-state"][value="confirmed_none"]').waitFor({ state: 'visible', timeout: 10000 });
  await page.getByText('ยังคำนวณกำไรสุทธิไม่ได้').waitFor({ timeout: 5000 });
  await page.check('input[name="fixed-cost-state"][value="confirmed_none"]');
  await page.getByText('กำไรสุทธิโดยประมาณ').waitFor({ timeout: 5000 });
  const confirmSave = page.waitForResponse((r) => r.request().method() === 'POST', { timeout: 10000 });
  await page.click('button:has-text("บันทึกส่วนนี้")');
  if (!(await confirmSave).ok()) throw new Error('confirming an empty fixed section returned a failing HTTP status');
  await page.locator('.form-message:not(:empty)').first().waitFor({ timeout: 10000 });
  await reloadOrDiagnose(page);
  await waitForHydratedValue(page, 'input[name="fixed-cost-state"][value="confirmed_none"]', (v) => v === true, 'refresh lost the confirmed-none answer');
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.getByText('ยืนยันแล้วว่าไม่มีค่าใช้จ่ายส่วนนี้').waitFor({ timeout: 10000 });

  // A separate season reaches final comparison so the responsive matrix can
  // measure both the editable draft and immutable result states.
  await navigateOrDiagnose(page, `${BASE}/plans/new`, null, () => page.goto(`${BASE}/plans/new`));
  await page.fill('input[name="season_year"]', '2571');
  await page.fill('input[name="name"]', 'สวนทดสอบผลจริง');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
  const comparisonPlanId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!comparisonPlanId) throw new Error('the comparison proof season was not created');
  for (const [step, value, action] of [
    ['production', '20000', 'ถัดไป'],
    ['price', '80', 'ถัดไป'],
    ['cost', '900000', 'ดูผลประมาณการ'],
  ]) {
    await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/quick/${step}$`));
    await page.fill('input[name="value"]', value);
    await page.click(`button:has-text("${action}")`);
  }
  await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/quick/result$`));
  await navigateOrDiagnose(page, `${BASE}/plans/${comparisonPlanId}/close`, null, () => page.goto(`${BASE}/plans/${comparisonPlanId}/close`));
  await page.fill('input[name="sellable_yield_kg"]', '18000');
  await page.fill('input[name="revenue"]', '1530000');
  await page.fill('input[name="total_cost"]', '990000');
  await page.fill('textarea[name="note"]', 'ผลผลิตน้อยกว่าคาด');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/close/review$`));
  await page.click('a:has-text("กลับไปแก้ผลจริง")');
  await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/close$`));
  if (await page.locator('input[name="total_cost"]').inputValue() !== '990000') {
    throw new Error('returning from review lost the persisted actual draft');
  }
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/close/review$`));
  await page.click('button:has-text("ยืนยันผลจริงและปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${comparisonPlanId}/comparison$`));
  await page
    .getByText('ต่ำกว่าประมาณการ 2,000.00 กก.', { exact: true })
    .waitFor({ state: 'visible', timeout: 10000 })
    .catch(() => {
      throw new Error('final comparison did not show the deterministic yield delta');
    });

  // Finalize the earlier 2569 draft with different values. History must then
  // compare the 2571 actual directly with 2569, name the skipped year, and use
  // the fixed transparent percentage rule rather than inventing 2570 data.
  await navigateOrDiagnose(page, `${BASE}/plans/${planId}/close`, null, () => page.goto(`${BASE}/plans/${planId}/close`));
  await page.fill('input[name="sellable_yield_kg"]', '16000');
  await page.fill('input[name="revenue"]', '1400000');
  await page.fill('input[name="total_cost"]', '950000');
  await page.fill('textarea[name="note"]', 'ปีฐานสำหรับประวัติ');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${planId}/close/review$`));
  await page.click('button:has-text("ยืนยันผลจริงและปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${planId}/comparison$`));

  await navigateOrDiagnose(page, `${BASE}/history`, null, () => page.goto(`${BASE}/history`));
  await page
    .getByText('กิโลที่ขายได้ สูงกว่าฤดูกาลก่อน 12.50%', { exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });
  await page
    .getByText('มีปีที่ข้ามระหว่างสองผลจริง ระบบเทียบเฉพาะปีที่แสดงและไม่ประมาณค่าปีที่หายไป', { exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });

  // Targets remain available, but only behind the explicitly optional
  // advanced area. Saving one and returning must retain the owner's value.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  const advancedTarget = page.locator('a:has-text("เป้าหมาย KPI")');
  if (await advancedTarget.isVisible()) {
    throw new Error('advanced targets appeared in the main journey before expansion');
  }
  await page.click('summary:has-text("การวางแผนขั้นสูง (ไม่บังคับ)")');
  await advancedTarget.click();
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/targets$`));
  const yieldTarget = page.locator('label:has-text("ผลผลิตต่อไร่") input');
  await yieldTarget.fill('2200');
  await page.click('button:has-text("บันทึกส่วนนี้")');
  await page.getByText('บันทึกแล้ว', { exact: true }).waitFor({ state: 'visible', timeout: 10000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.click('summary:has-text("การวางแผนขั้นสูง (ไม่บังคับ)")');
  await page.click('a:has-text("เป้าหมาย KPI")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/targets$`));
  if (await page.locator('label:has-text("ผลผลิตต่อไร่") input').inputValue() !== '2200') {
    throw new Error('advanced target did not persist after leaving and returning');
  }

  // An asset is entered once, starts excluded, and affects only a season the
  // owner explicitly includes it in. Starting capital stays visibly separate.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.click('summary:has-text("การวางแผนขั้นสูง (ไม่บังคับ)")');
  await page.click('a:has-text("ของที่ใช้หลายปี")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/assets$`));
  await page.click('summary:has-text("+ เพิ่มของที่ใช้หลายปี")');
  const create = page.locator('details.asset-create');
  await create.locator('input[name="name"]').fill('ระบบน้ำกลางสวน');
  await create.locator('input[name="original_cost"]').fill('100000');
  await create.locator('input[name="start_year"]').fill('2568');
  await create.locator('input[name="useful_life_years"]').fill('5');
  // Each assets action must show its result without a reload: the owner saw
  // "รวมในฤดูนี้" save and the page stay as it was until a manual refresh.
  await submitInPlace(page, create.locator('button:has-text("บันทึกของชิ้นนี้")'), 'asset creation');
  await page.getByText('ระบบน้ำกลางสวน', { exact: true }).waitFor({ state: 'visible', timeout: 10000 });
  await page.getByText('ยังไม่รวม', { exact: true }).waitFor({ state: 'visible' });
  await assertNoReload(page, 'asset creation');

  await page.getByText('ระบบจะไม่เดาหรือลบรายการเดิมให้', { exact: false }).waitFor({ state: 'visible' });
  await submitInPlace(page, page.locator('button:has-text("รวมในฤดูนี้")'), 'asset inclusion');
  await page.locator('.asset-row .status', { hasText: 'รวมในฤดูนี้' }).waitFor({ state: 'visible', timeout: 10000 });
  await page.locator('.asset-totals', { hasText: '20,000.00 บาท/ปี' }).waitFor({ state: 'visible', timeout: 10000 });
  await page.locator('button:has-text("เอาออกจากฤดูนี้")').waitFor({ state: 'visible' });
  await assertNoReload(page, 'asset inclusion');

  await page.fill('input[name="starting_capital"]', '50000');
  await submitInPlace(page, page.locator('button:has-text("บันทึกเงินก้อนตั้งต้น")'), 'starting capital save');
  await page.locator('.asset-totals', { hasText: '50,000.00' }).waitFor({ state: 'visible', timeout: 10000 });
  await assertNoReload(page, 'starting capital save');
  if (await page.locator('input[name="starting_capital"]').inputValue() !== '50000') {
    throw new Error('starting capital did not persist independently');
  }

  // Including an asset must be visible where the owner would look for its
  // cost: the fixed-costs page lists it as a read-only row in its own group
  // under the rows the owner types, with one total, and the hub names it too.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/fixed-costs`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/fixed-costs`));
  const manualGroup = page.locator('.cost-group-manual');
  const assetGroup = page.locator('.cost-group-assets');
  const fixedTotal = page.locator('.fixed-cost-total');
  await manualGroup.getByText('กรอกเอง', { exact: true }).waitFor({ timeout: 10000 });
  await assetGroup.getByText('ของที่ใช้หลายปี · ค่าเสื่อม', { exact: true }).waitFor({ timeout: 5000 });
  await assetGroup.locator('.asset-cost-row', { hasText: 'ระบบน้ำกลางสวน' }).getByText('20,000.00 บาท/ปี').waitFor({ timeout: 5000 });
  await assetGroup.locator('a[href$="/assets"]').first().waitFor({ state: 'visible' });
  // The unknown / confirmed-none question belongs to the manual group only.
  // The manual group was confirmed empty earlier, so asset-only fixed cost is
  // a known figure; answering "ยังไม่รู้" makes the total unknown again even
  // though the depreciation stays listed, and confirming brings it back.
  await manualGroup.locator('input[name="fixed-cost-state"]').first().waitFor({ state: 'attached' });
  if (await assetGroup.locator('input[name="fixed-cost-state"]').count()) {
    throw new Error('the section-state question leaked into the asset group');
  }
  await fixedTotal.locator('.cost-total-amount', { hasText: '20,000.00 บาท/ปี' }).waitFor({ timeout: 5000 });
  await page.locator('label:has(input[name="fixed-cost-state"][value="unknown"])').click();
  await fixedTotal.locator('.cost-total-amount', { hasText: 'ยังไม่รู้' }).waitFor({ timeout: 5000 });
  await fixedTotal.locator('.cost-total-note', { hasText: 'รวมอยู่แล้ว แต่ยอดรวมยังไม่รู้' }).waitFor({ timeout: 5000 });
  await page.getByText('ยังคำนวณกำไรสุทธิไม่ได้', { exact: true }).waitFor({ timeout: 5000 });
  await page.locator('label:has(input[name="fixed-cost-state"][value="confirmed_none"])').click();
  await fixedTotal.locator('.cost-total-amount', { hasText: '20,000.00 บาท/ปี' }).waitFor({ timeout: 5000 });
  await page.getByText('กำไรสุทธิโดยประมาณ', { exact: false }).waitFor({ timeout: 5000 });
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}`));
  await page.getByText(/ค่าเสื่อมของที่เลือกไว้รวมแล้ว|ยืนยันแล้วว่ามีเฉพาะค่าเสื่อมของที่เลือกไว้|พอคำนวณค่าใช้จ่ายประจำแล้ว/).first().waitFor({ timeout: 10000 });

  // The same owner asset appears in another season without re-entry, but is
  // excluded there until the owner makes a second explicit choice.
  await navigateOrDiagnose(page, `${BASE}/plans/new`, null, () => page.goto(`${BASE}/plans/new`));
  await page.fill('input[name="season_year"]', '2572');
  await page.fill('input[name="name"]', 'สวนทดสอบสินทรัพย์เปิด');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
  const assetOpenPlanId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!assetOpenPlanId) throw new Error('the open asset proof season was not created');
  await navigateOrDiagnose(page, `${BASE}/plans/${assetOpenPlanId}`, null, () => page.goto(`${BASE}/plans/${assetOpenPlanId}`));
  await submitAndReload(page, page.locator('button:has-text("เปลี่ยนเป็นแผนละเอียด")'), 'asset proof mode switch');
  await navigateOrDiagnose(page, `${BASE}/plans/${assetOpenPlanId}/assets`, null, () => page.goto(`${BASE}/plans/${assetOpenPlanId}/assets`));
  await page.getByText('ระบบน้ำกลางสวน', { exact: true }).waitFor({ state: 'visible' });
  await page.getByText('ยังไม่รวม', { exact: true }).waitFor({ state: 'visible' });

  // Keep an editable actual draft for the close and review layout matrix.
  await navigateOrDiagnose(page, `${BASE}/plans/${assetOpenPlanId}/close`, null, () => page.goto(`${BASE}/plans/${assetOpenPlanId}/close`));
  await page.fill('input[name="sellable_yield_kg"]', '17000');
  await page.fill('input[name="revenue"]', '1450000');
  await page.fill('input[name="total_cost"]', '910000');
  await page.fill('textarea[name="note"]', 'ฉบับรอตรวจทาน');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${assetOpenPlanId}/close/review$`));

  // Closing freezes the selected facts and contribution. The closed asset page
  // must remain readable and expose no edit or selection controls.
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/close`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/close`));
  await page.fill('input[name="sellable_yield_kg"]', '18000');
  await page.fill('input[name="revenue"]', '1530000');
  await page.fill('input[name="total_cost"]', '990000');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/close/review$`));
  await page.click('button:has-text("ยืนยันผลจริงและปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/comparison$`));
  await navigateOrDiagnose(page, `${BASE}/plans/${detailedPlanId}/assets`, null, () => page.goto(`${BASE}/plans/${detailedPlanId}/assets`));
  await page.getByText('ข้อมูลนี้ถูกเก็บพร้อมตอนปิดฤดู · แก้ไขไม่ได้', { exact: true }).waitFor({ state: 'visible' });
  if (await page.locator('button:has-text("เอาออกจากฤดูนี้")').count()) {
    throw new Error('closed asset snapshot still exposed a selection control');
  }

  const cookies = await context.cookies();
  await context.close();
  return { cookies, planId, detailedPlanId, comparisonPlanId, assetOpenPlanId };
}

/** No sheet may be left standing between measurements. */
async function ensureClosed(page) {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    if (!(await page.locator('details[open] > .sheet').count())) return true;
    const close = page.locator('details[open] > .sheet > .sheet-close').first();
    await close.click({ timeout: 3000 }).catch(() => {});
    await page.waitForTimeout(120);
  }
  return !(await page.locator('details[open] > .sheet').count());
}

/** Open each visible ⓘ in turn and measure the page while it is open. */
async function openEachExplanation(page, size, where) {
  const selector = 'details.figure-explanation:visible > summary, details.live-explanation:visible > summary';
  const count = await page.locator(selector).count();
  for (let index = 0; index < count; index += 1) {
    const summary = page.locator(selector).nth(index);
    if (!(await summary.isVisible())) continue;
    // Measuring scrolls the page, so bring the control back before clicking it.
    // A control that cannot be opened is reported, never thrown, so one flaky
    // interaction cannot turn a proof into a crash with no findings.
    try {
      await summary.scrollIntoViewIfNeeded({ timeout: 5000 });
      await summary.click({ timeout: 5000 });
    } catch {
      fail(`${where} @${size.name}px ⓘ#${index + 1}`, 'the explanation could not be opened');
      continue;
    }
    // The sheet animates up over 180ms; measuring sooner measures it in flight.
    await page.waitForTimeout(400);
    assess(`${where} @${size.name}px ⓘ#${index + 1}`, await inspect(page, size.width), size.width);

    // Close the way a reader does. The icon is behind the backdrop once the
    // sheet is up, so an explanation that can only be closed by finding it
    // again is a defect, not a detail of the harness.
    const close = page.locator('details[open] > .sheet > .sheet-close');
    if (await close.count()) {
      await close.first().click({ timeout: 5000 }).catch(() => {});
    }
    await page.waitForTimeout(200);
    if (await page.locator('details[open] > .sheet').count()) {
      fail(`${where} @${size.name}px ⓘ#${index + 1}`, 'the explanation would not close');
      await reloadOrDiagnose(page);
    }
  }
}

const browser = await chromium.launch({ channel: 'chrome' });
try {
  const { cookies, planId, detailedPlanId, comparisonPlanId, assetOpenPlanId } = await signIn(browser);
  const routes = [
    // The entry screen is for a visitor who is not signed in; a signed-in
    // visit to either address is sent to /plans and would measure that page.
    ['home', `${BASE}/`, { signedOut: true }],
    ['register', `${BASE}/register`],
    ['login', `${BASE}/login`, { signedOut: true }],
    ['verify-recovery', `${BASE}/verify-email?invalid=1`],
    ['forgot-password', `${BASE}/forgot-password`],
    ['reset-recovery', `${BASE}/reset-password?token=invalid`],
    ['demo', `${BASE}/demo`],
    ['new-season', `${BASE}/plans/new`],
    ['plans', `${BASE}/plans`],
    ['quick-hub', `${BASE}/plans/${planId}`],
    ['quick-production', `${BASE}/plans/${planId}/quick/production`],
    ['quick-price', `${BASE}/plans/${planId}/quick/price`],
    ['quick-cost', `${BASE}/plans/${planId}/quick/cost`],
    ['quick-result', `${BASE}/plans/${planId}/quick/result`],
    ['actual-entry', `${BASE}/plans/${assetOpenPlanId}/close`],
    ['actual-review', `${BASE}/plans/${assetOpenPlanId}/close/review`],
    ['actual-comparison', `${BASE}/plans/${comparisonPlanId}/comparison`],
    ['history', `${BASE}/history?from=${comparisonPlanId}`],
    ['detailed-hub', `${BASE}/plans/${detailedPlanId}`],
    ['dashboard', `${BASE}/plans/${detailedPlanId}/dashboard`],
    ['analysis', `${BASE}/plans/${detailedPlanId}/analysis`],
    ['production', `${BASE}/plans/${detailedPlanId}/production`],
    ['market', `${BASE}/plans/${detailedPlanId}/market`],
    ['expenses', `${BASE}/plans/${detailedPlanId}/expenses`],
    ['variable-costs', `${BASE}/plans/${detailedPlanId}/variable-costs`],
    ['fixed-costs', `${BASE}/plans/${detailedPlanId}/fixed-costs`],
    ['tax-deductions', `${BASE}/plans/${detailedPlanId}/tax-deductions`],
    ['health', `${BASE}/plans/${detailedPlanId}/health`],
    ['targets-advanced', `${BASE}/plans/${detailedPlanId}/targets`],
    ['assets-closed', `${BASE}/plans/${detailedPlanId}/assets`],
    ['assets-open', `${BASE}/plans/${assetOpenPlanId}/assets`],
    // Error, empty, and unknown-route states an owner can land on.
    ['unknown-route', `${BASE}/no-such-page`],
    ['season-not-found', `${BASE}/plans/999999999`],
    ['section-not-found', `${BASE}/plans/${detailedPlanId}/no-such-section`],
    ['comparison-before-close', `${BASE}/plans/${assetOpenPlanId}/comparison`],
    ['review-without-draft', `${BASE}/plans/${detailedPlanId}/close/review`],
    ['entry-after-close', `${BASE}/plans/${comparisonPlanId}/close`],
    ['review-after-close', `${BASE}/plans/${comparisonPlanId}/close/review`],
    // The loading state: a client-side navigation whose data is held back,
    // so the Suspense fallback is on screen when it is measured.
    ['hub-loading', `${BASE}/plans`],
  ];

  for (const size of WIDTHS) {
    const context = await browser.newContext({
      viewport: { width: size.width, height: size.height },
      isMobile: true,
      hasTouch: true,
      deviceScaleFactor: 2,
    });
    await context.addCookies(cookies);
    const signedOut = await browser.newContext({
      viewport: { width: size.width, height: size.height },
      isMobile: true,
      hasTouch: true,
      deviceScaleFactor: 2,
    });

    for (const [routeName, url, options = {}] of routes) {
      // A fresh page makes every route measurement independent and prevents a
      // long matrix from retaining old hydrated runtimes in one renderer.
      const page = await (options.signedOut ? signedOut : context).newPage();
      try {
        if (routeName === 'hub-loading') {
          await page.route('**/api/load_plan*', async (route) => {
            await new Promise((resolve) => setTimeout(resolve, 2500));
            await route.continue();
          });
          // The fallback shows only on a client-side navigation, which needs
          // hydration first. The wasm may come from the context's cache, so
          // its network response is not a reliable signal; give hydration a
          // moment and retry the click once if the navigation was a full load.
          let shown = false;
          for (let attempt = 0; attempt < 2 && !shown; attempt += 1) {
            await gotoOrDiagnose(page, url, cookies);
            await Promise.race([
              page.waitForResponse((response) => response.url().endsWith('.wasm'), { timeout: 8000 }).catch(() => {}),
              page.waitForTimeout(8000),
            ]);
            await page.waitForTimeout(1000);
            await page.locator(`a[href="/plans/${detailedPlanId}"]`).first().click();
            shown = await page
              .getByText('กำลังอ่านฤดูกาล…')
              .waitFor({ state: 'visible', timeout: 4000 })
              .then(() => true)
              .catch(() => false);
          }
          if (!shown) throw new Error(`${size.name}px ${routeName}: the loading fallback never appeared`);
          assess(`${size.name}px ${routeName}`, await inspect(page, size.width), size.width);
          await page.unroute('**/api/load_plan*');
          continue;
        }
        await gotoOrDiagnose(page, url, cookies);
        await page.waitForTimeout(250);
        assess(`${size.name}px ${routeName}`, await inspect(page, size.width), size.width);

        // Every explanation, one at a time: an open card is the state that failed.
        // Only visible ones — a panel carrying `hidden` is not on screen to tap.
        await openEachExplanation(page, size, routeName);

        if (routeName === 'analysis') {
          for (const tab of ['ตรวจสอบ', 'ภาษี', 'สถานการณ์']) {
            const button = page.locator(`button[role="tab"]:has-text("${tab}")`);
            if (await button.count()) {
              await ensureClosed(page);
              await button.click({ timeout: 10000 });
              await page.waitForTimeout(200);
              assess(`${size.name}px analysis:${tab}`, await inspect(page, size.width), size.width);
              await openEachExplanation(page, size, `analysis:${tab}`);
            }
          }
          await ensureClosed(page);
          const table = page.locator('details.scenario-table:visible > summary');
          if (await table.count()) {
            await table.click({ timeout: 10000 });
            await page.waitForTimeout(250);
            assess(`${size.name}px analysis:table`, await inspect(page, size.width), size.width);
          }
        }
      } finally {
        await page.close();
      }
    }
    console.log(`Responsive proof measured ${routes.length} routes at ${size.name}px`);
    await context.close();
    await signedOut.close();
  }
} finally {
  await browser.close();
}

if (failures.length) {
  console.error(`\nResponsive proof failed with ${failures.length} finding(s):\n`);
  for (const failure of failures) console.error(`  ${failure}`);
  process.exit(1);
}
console.log(`Responsive proof passed at ${WIDTHS.map((w) => w.name).join(', ')}px`);

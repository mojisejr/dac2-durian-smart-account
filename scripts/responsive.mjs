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
          clipped.push({ el: name(el), text: (el.textContent || '').trim().slice(0, 40) });
        }
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
    fail(where, `${item.el} clips its own text — "${item.text}"`);
  }
  for (const item of report.small.slice(0, 4)) {
    fail(where, `${item.el} is ${item.w}x${item.h}, under the 48px rule — "${item.text}"`);
  }
  for (const item of report.covered.slice(0, 4)) {
    fail(where, `${item.el} is covered by ${item.by} and cannot be tapped — "${item.text}"`);
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

async function signIn(browser) {
  const email = `responsive${Date.now()}@dac2.local`;
  const password = 'CorrectHorse123!';
  const context = await browser.newContext({ viewport: { width: 412, height: 915 } });
  const page = await context.newPage();

  await page.goto(`${BASE}/register`);
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
  await page.goto(link);

  await page.goto(`${BASE}/login`);
  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  await page.waitForTimeout(1200);

  await page.goto(`${BASE}/plans`);
  await page.click('button:has-text("เปิดแผนตัวอย่าง")');
  await page.waitForTimeout(1500);
  await page.goto(`${BASE}/plans`);
  const href = await page.locator('a[href^="/plans/"]').first().getAttribute('href');
  const planId = href?.split('/')[2];
  if (!planId) throw new Error('the sample plan was not created');

  const cookies = await context.cookies();
  await context.close();
  return { cookies, planId };
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
      await page.reload({ waitUntil: 'networkidle' });
    }
  }
}

const browser = await chromium.launch({ channel: 'chrome' });
try {
  const { cookies, planId } = await signIn(browser);
  const routes = [
    ['plans', `${BASE}/plans`],
    ['hub', `${BASE}/plans/${planId}`],
    ['dashboard', `${BASE}/plans/${planId}/dashboard`],
    ['analysis', `${BASE}/plans/${planId}/analysis`],
    ['production', `${BASE}/plans/${planId}/production`],
    ['health', `${BASE}/plans/${planId}/health`],
  ];

  for (const size of WIDTHS) {
    const context = await browser.newContext({
      viewport: { width: size.width, height: size.height },
      isMobile: true,
      hasTouch: true,
      deviceScaleFactor: 2,
    });
    await context.addCookies(cookies);
    const page = await context.newPage();

    for (const [routeName, url] of routes) {
      await page.goto(url, { waitUntil: 'networkidle' });
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
    }
    await context.close();
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

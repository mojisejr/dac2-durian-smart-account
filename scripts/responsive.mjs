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
  await page.reload({ waitUntil: 'domcontentloaded' });
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

async function signIn(browser) {
  const email = 'responsive-proof@dac2.local';
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

  // The workbook sample is a browser-only sandbox. Editing and leaving it must
  // not create a season row.
  await page.goto(`${BASE}/demo`);
  await page.getByLabel('ราคาเกรดแรก').fill('120');
  await page.click('button:has-text("คืนค่าตัวอย่าง")');
  await page.goto(`${BASE}/plans`);
  if (await page.locator('.plan-list-item').count()) {
    throw new Error('the browser-only demonstration created a stored season');
  }

  await page.goto(`${BASE}/plans/new`);
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

  // Leaving after one answer and returning through the hub must resume at the
  // first unanswered question rather than restarting or inventing a result.
  await page.goto(`${BASE}/plans/${planId}`);
  await page.click('a:has-text("ทำประมาณการต่อ")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/price$`));
  await page.fill('input[name="value"]', '80');
  await page.click('button:has-text("ถัดไป")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/cost$`));
  await page.goBack({ waitUntil: 'domcontentloaded' });
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/price$`));
  if (await page.locator('input[name="value"]').inputValue() !== '80') {
    throw new Error('browser back lost the persisted quick price');
  }
  await page.click('button:has-text("ถัดไป")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/cost$`));
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.fill('input[name="value"]', '900000');
  await page.click('button:has-text("ดูผลประมาณการ")');
  await page.waitForURL(new RegExp(`/plans/${planId}/quick/result$`));

  // Save a reviewable actual draft. Leaving review must keep the season open
  // and the draft available when the owner returns.
  await page.goto(`${BASE}/plans/${planId}/close`);
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
  await page.goto(`${BASE}/plans/${planId}/close/review`);

  // Keep a separate detailed-mode season in the same account so this proof
  // covers the new quick surfaces without dropping the pre-existing detailed
  // dashboard, analysis, and long-form input regression.
  await page.goto(`${BASE}/plans/new`);
  await page.fill('input[name="season_year"]', '2570');
  await page.fill('input[name="name"]', 'สวนทดสอบละเอียด');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
  const detailedPlanId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!detailedPlanId) throw new Error('the detailed proof season was not created');
  await page.goto(`${BASE}/plans/${detailedPlanId}`);
  const modeResponse = page.waitForResponse(
    (response) => response.request().method() === 'POST',
    { timeout: 10000 },
  );
  await page.click('button:has-text("เปลี่ยนเป็นแผนละเอียด")');
  const modeResult = await modeResponse;
  if (!modeResult.ok()) {
    throw new Error(`switching to detailed mode returned HTTP ${modeResult.status()}`);
  }
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page
    .getByRole('heading', { name: 'แผนละเอียด', exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });

  // A separate season reaches final comparison so the responsive matrix can
  // measure both the editable draft and immutable result states.
  await page.goto(`${BASE}/plans/new`);
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
  await page.goto(`${BASE}/plans/${comparisonPlanId}/close`);
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
  await page.goto(`${BASE}/plans/${planId}/close`);
  await page.fill('input[name="sellable_yield_kg"]', '16000');
  await page.fill('input[name="revenue"]', '1400000');
  await page.fill('input[name="total_cost"]', '950000');
  await page.fill('textarea[name="note"]', 'ปีฐานสำหรับประวัติ');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${planId}/close/review$`));
  await page.click('button:has-text("ยืนยันผลจริงและปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${planId}/comparison$`));

  await page.goto(`${BASE}/history`);
  await page
    .getByText('ผลผลิตที่ขายได้ สูงกว่าฤดูกาลก่อน 12.50%', { exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });
  await page
    .getByText('มีปีที่ข้ามระหว่างสองผลจริง ระบบเทียบเฉพาะปีที่แสดงและไม่ประมาณค่าปีที่หายไป', { exact: true })
    .waitFor({ state: 'visible', timeout: 10000 });

  // Targets remain available, but only behind the explicitly optional
  // advanced area. Saving one and returning must retain the owner's value.
  await page.goto(`${BASE}/plans/${detailedPlanId}`);
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
  await page.goto(`${BASE}/plans/${detailedPlanId}`);
  await page.click('summary:has-text("การวางแผนขั้นสูง (ไม่บังคับ)")');
  await page.click('a:has-text("เป้าหมาย KPI")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/targets$`));
  if (await page.locator('label:has-text("ผลผลิตต่อไร่") input').inputValue() !== '2200') {
    throw new Error('advanced target did not persist after leaving and returning');
  }

  // An asset is entered once, starts excluded, and affects only a season the
  // owner explicitly includes it in. Starting capital stays visibly separate.
  await page.goto(`${BASE}/plans/${detailedPlanId}`);
  await page.click('summary:has-text("การวางแผนขั้นสูง (ไม่บังคับ)")');
  await page.click('a:has-text("สินทรัพย์และเงินลงทุน")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/assets$`));
  await page.click('summary:has-text("+ เพิ่มสินทรัพย์")');
  const create = page.locator('details.asset-create');
  await create.locator('input[name="name"]').fill('ระบบน้ำกลางสวน');
  await create.locator('input[name="original_cost"]').fill('100000');
  await create.locator('input[name="start_year"]').fill('2568');
  await create.locator('input[name="useful_life_years"]').fill('5');
  await submitAndReload(page, create.locator('button:has-text("บันทึกสินทรัพย์")'), 'asset creation');
  await page.getByText('ยังไม่รวม', { exact: true }).waitFor({ state: 'visible' });
  await page.getByText('ระบบจะไม่เดาหรือลบรายการเดิมให้', { exact: false }).waitFor({ state: 'visible' });
  await submitAndReload(page, page.locator('button:has-text("รวมในฤดูนี้")'), 'asset inclusion');
  await page.getByText('รวมในฤดูนี้', { exact: true }).waitFor({ state: 'visible' });
  await page.fill('input[name="starting_capital"]', '50000');
  await submitAndReload(page, page.locator('button:has-text("บันทึกเงินทุนเริ่มต้น")'), 'starting capital save');
  if (await page.locator('input[name="starting_capital"]').inputValue() !== '50000') {
    throw new Error('starting capital did not persist independently');
  }

  // The same owner asset appears in another season without re-entry, but is
  // excluded there until the owner makes a second explicit choice.
  await page.goto(`${BASE}/plans/new`);
  await page.fill('input[name="season_year"]', '2572');
  await page.fill('input[name="name"]', 'สวนทดสอบสินทรัพย์เปิด');
  await page.click('button:has-text("สร้างฤดูกาล")');
  await page.waitForURL(/\/plans\/\d+\/quick\/production$/);
  const assetOpenPlanId = page.url().match(/\/plans\/(\d+)\/quick\/production$/)?.[1];
  if (!assetOpenPlanId) throw new Error('the open asset proof season was not created');
  await page.goto(`${BASE}/plans/${assetOpenPlanId}`);
  await submitAndReload(page, page.locator('button:has-text("เปลี่ยนเป็นแผนละเอียด")'), 'asset proof mode switch');
  await page.goto(`${BASE}/plans/${assetOpenPlanId}/assets`);
  await page.getByText('ระบบน้ำกลางสวน', { exact: true }).waitFor({ state: 'visible' });
  await page.getByText('ยังไม่รวม', { exact: true }).waitFor({ state: 'visible' });

  // Closing freezes the selected facts and contribution. The closed asset page
  // must remain readable and expose no edit or selection controls.
  await page.goto(`${BASE}/plans/${detailedPlanId}/close`);
  await page.fill('input[name="sellable_yield_kg"]', '18000');
  await page.fill('input[name="revenue"]', '1530000');
  await page.fill('input[name="total_cost"]', '990000');
  await page.click('button:has-text("บันทึกและตรวจทาน")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/close/review$`));
  await page.click('button:has-text("ยืนยันผลจริงและปิดฤดูกาล")');
  await page.waitForURL(new RegExp(`/plans/${detailedPlanId}/comparison$`));
  await page.goto(`${BASE}/plans/${detailedPlanId}/assets`);
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
      await page.reload({ waitUntil: 'domcontentloaded' });
    }
  }
}

const browser = await chromium.launch({ channel: 'chrome' });
try {
  const { cookies, planId, detailedPlanId, comparisonPlanId, assetOpenPlanId } = await signIn(browser);
  const routes = [
    ['demo', `${BASE}/demo`],
    ['new-season', `${BASE}/plans/new`],
    ['plans', `${BASE}/plans`],
    ['quick-hub', `${BASE}/plans/${planId}`],
    ['quick-production', `${BASE}/plans/${planId}/quick/production`],
    ['quick-price', `${BASE}/plans/${planId}/quick/price`],
    ['quick-cost', `${BASE}/plans/${planId}/quick/cost`],
    ['quick-result', `${BASE}/plans/${planId}/quick/result`],
    ['actual-entry', `${BASE}/plans/${planId}/close`],
    ['actual-review', `${BASE}/plans/${planId}/close/review`],
    ['actual-comparison', `${BASE}/plans/${comparisonPlanId}/comparison`],
    ['history', `${BASE}/history?from=${comparisonPlanId}`],
    ['detailed-hub', `${BASE}/plans/${detailedPlanId}`],
    ['dashboard', `${BASE}/plans/${detailedPlanId}/dashboard`],
    ['analysis', `${BASE}/plans/${detailedPlanId}/analysis`],
    ['production', `${BASE}/plans/${detailedPlanId}/production`],
    ['health', `${BASE}/plans/${detailedPlanId}/health`],
    ['targets-advanced', `${BASE}/plans/${detailedPlanId}/targets`],
    ['assets-closed', `${BASE}/plans/${detailedPlanId}/assets`],
    ['assets-open', `${BASE}/plans/${assetOpenPlanId}/assets`],
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
      // A full document navigation keeps route measurements independent without
      // accumulating twenty live page runtimes inside one phone viewport.
      await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
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
    await page.close();
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

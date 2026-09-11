# DAC2 — Design

**Status:** draft, revision 0.4
**Home:** this file moves to the application repository root in slice 1. There is
one copy of it, never two.

## How this document is used

This is the owner's surface for deciding how the application looks and behaves.
The owner edits it; an agent implementing a UI slice follows it and does not
invent an alternative. Where this document and a plan disagree, this document
wins on appearance and interaction, and the plan wins on scope.

If an agent needs something this document does not cover, it adds the question
to **Open questions** rather than choosing silently.

Prose and rationale are in English, matching the rest of the repository. Every
string a user sees is written here in Thai, verbatim, because that is what ships.

## Who uses this, and where

An orchard owner, typically in their thirties or older, fluent with phone
applications and used to Shopee, LINE, food delivery, and mobile banking. They
do not need an interface explained to them, and padding the screen with
instructions insults them.

They are also standing in an orchard. Sun on the screen, one hand on the phone,
the other hand doing something else, possibly dirty or wet.

Those two facts pull in different directions and both are binding. Fluency
allows modern density, gestures, bottom sheets, and few words. The orchard
requires contrast beyond the accessible minimum, large touch targets, and a
layout that works with one thumb. Neither excuses the other.

## Principles

1. **The number is the interface.** Every screen exists to put one figure in
   front of the owner. Decide what that figure is before laying anything out.
2. **Answer before detail.** The conclusion goes at the top at the largest size.
   Supporting figures come after, and inputs come last.
3. **Never make them wait to see the effect of a number they typed.** The
   calculation runs in the browser. A figure that changes should already have
   changed.
4. **Say the uncomfortable thing.** A plan with good profit and poor
   productivity must read as both. The interface does not flatter.
5. **Familiar beats clever.** If Shopee or a banking application has already
   taught this interaction to the user, use their version of it.
6. **Nothing moves to attract attention.** Motion confirms what the user just
   did. It never advertises.

## Colour

Two themes, light and dark, both selectable and both defaulting to the device
setting. **Light mode is designed to be read in direct sunlight**, so there is no
third high-contrast mode; the ordinary one already is.

Contrast ratios below were computed against the WCAG formula, not estimated. The
floor for this project is **6:1 for any text**, above the 4.5:1 that AA requires,
because glare consumes real contrast that a specification cannot see.

### Light

| Token | Value | On `bg` | Grade |
|---|---|---|---|
| `bg` | `#F7F6F2` | — | warm off-white; less glare than pure white |
| `surface` | `#FFFFFF` | — | cards and sheets |
| `text` | `#101418` | 17.11:1 | AAA |
| `text-muted` | `#434E5A` | 7.84:1 | AAA |
| `primary` | `#0F5C3A` | 7.43:1 | AAA |
| `good` | `#0B6236` | 6.90:1 | AA |
| `warn` | `#8A4F00` | 6.07:1 | AA |
| `bad` | `#A6231C` | 6.75:1 | AA |
| `border` | `#DEDCD5` | 1.27:1 | decorative dividers only |
| `border-strong` | `#6E7681` | 4.25:1 | input outlines; clears the 3:1 floor |

White text on `primary`, `good`, and `bad` fills reaches 8.04:1, 7.46:1, and
7.30:1.

### Dark

| Token | Value | On `bg` | Grade |
|---|---|---|---|
| `bg` | `#0D1214` | — | not pure black; pure black smears on OLED while scrolling |
| `surface` | `#161D21` | — | elevation is lightness, not shadow |
| `text` | `#EDF2F4` | 16.71:1 | AAA |
| `text-muted` | `#A0AFB7` | 8.36:1 | AAA |
| `primary` | `#4ADE9A` | 10.96:1 | AAA |
| `good` | `#3FD68A` | 10.05:1 | AAA |
| `warn` | `#E8A33D` | 8.74:1 | AAA |
| `bad` | `#FF7A6E` | 7.42:1 | AAA |
| `border` | `#2A343A` | 1.48:1 | decorative dividers only |
| `border-strong` | `#5C6A73` | 3.38:1 | input outlines |

### Rules

- **Colour is never the only signal.** Every good or bad state carries its word
  as well: `ถึงเป้า` or `ปรับปรุง`, `กำไร` or `ขาดทุน`. Roughly one Thai man in
  twelve cannot separate the greens from the reds. The source workbook already
  got this right and the application keeps it.
- Money figures are painted with `text`, not with `primary`. A number is data,
  not decoration. `good` and `bad` are reserved for a verdict about a number.
- `warn` means "look at this", not "something broke".
- A theme is a set of token values. No component names a hex code.

## Type

**Sarabun** throughout, self-hosted, no third-party font CDN. Sarabun is a looped
Thai face drawn for legibility at small sizes with a generous x-height and low
stroke contrast, which is what a screen in sunlight needs. It is also the Thai
letterform this user has seen more than any other. Its status as the government
document standard since 2011 is why it is everywhere; it is not why it was
chosen here.

**Thai needs more line height than Latin.** Body text sits at **1.6**, never
below 1.55, because Thai stacks vowels above and tone marks above those. At 1.4
the marks collide and the text stops being readable before it stops looking fine.

| Role | Size | Weight | Line height | Use |
|---|---|---|---|---|
| `hero` | 40 | 700 | 1.2 | the one figure a screen exists for |
| `title` | 24 | 600 | 1.3 | screen title |
| `section` | 18 | 600 | 1.4 | card heading |
| `body` | 16 | 400 | 1.6 | everything ordinary; never smaller for content |
| `label` | 14 | 500 | 1.5 | field labels, tab labels |
| `caption` | 13 | 400 | 1.5 | units, footnotes, the tax disclaimer |

Nothing user-facing is set below 13. A number the owner must act on is never
below `body`.

**Two decimal places, everywhere a figure is shown.** `834,600.00 บาท`,
`40.66 บาท/กก.`, `119.23%`. Rounding is presentation only: the calculation keeps
full precision and a rounded figure is never fed into another calculation.

**Figures are tabular.** `font-variant-numeric: tabular-nums` everywhere a number
appears in a column, so digits align down the page. Thousands separators always.
Currency is written `฿` before or `บาท` after, consistently, never both.

## Space, shape, elevation

- Spacing scale: 4, 8, 12, 16, 24, 32, 48. Nothing between.
- Radius: **8** for buttons and inputs, **14** for cards and sheets, **999** for
  badges and pills only. Not above 16 for a container; it reads as a toy.
- **Every interactive target is at least 48 by 48**, including a row in a list
  and an icon button. This is not negotiable and it is not about age.
- Elevation is a 1px `border` plus a barely visible shadow in light mode, and a
  lighter `surface` in dark mode. No coloured or large shadows.
- Screen padding is 16 on the sides. Content is a single column. There is no
  desktop layout in the first release; a wide screen gets the same column,
  centred, capped at 480.

## Motion

- 120–180ms for a state change. 220–280ms for a screen or sheet transition.
  Nothing exceeds 300ms.
- Easing `cubic-bezier(0.2, 0, 0, 1)`. No bounce, no spring, no overshoot.
- **A number never counts up.** A figure that animates while the owner is typing
  is noise. It changes, and the row behind it holds a 150ms tint to say which
  figure just moved.
- No skeletons and no shimmer. A pending region fades opacity.
- `prefers-reduced-motion: reduce` disables every transition. This is
  implemented, not intended.

## Components

**Number field.** The most used control in the application. Label above, unit
inside on the right (`กก.`, `บาท`, `ไร่`, `%`), `inputmode="decimal"`, 56 high,
thousands separators applied when focus leaves. Invalid values keep what the
owner typed and explain underneath in `bad`; the field is never cleared for them.

**Live calculation line.** Pinned above the tab bar on every input screen. It is
a compact `surface` row with a quiet border, not a dark result card. It says
`กำไรสุทธิโดยประมาณ` and its value, or `ยังคำนวณกำไรสุทธิไม่ได้`, and updates
as the owner types because the calculation runs locally. It carries no heading
such as `ยอดรวมสด` and no explanation control: this is feedback for the current
task, not a second destination competing with the form.

**Stat card.** A label, a figure at `hero` or `title`, a unit, and where a target
exists, the verdict word beside it in `good` or `warn`.

**List row.** 56 high minimum, label left, value right, chevron if it opens.
Whole row is the target, not the chevron.

**Explain button.** A 24px `ⓘ` beside any figure that carries meaning, with a
48px target around it. Tapping opens a bottom sheet answering four questions in
this order, always the same order:

```
จุดคุ้มทุน                                    ✕
─────────────────────────────────────────────
คืออะไร
ปริมาณที่ต้องขายให้ได้ ก่อนจะเริ่มมีกำไร

ใช้ยังไง
เทียบกับผลผลิตที่คาดว่าจะได้ ถ้าผลผลิตสูงกว่า
จุดคุ้มทุนมาก แปลว่ามีที่ให้พลาดได้เยอะ

ทำไมต้องมี
มันบอกว่าราคาตกได้แค่ไหนก่อนจะขาดทุน
ใช้ตัดสินใจตอนล้งเสนอราคาได้ทันที

ไม่ใส่ได้ไหม
คำนวณให้เองจากต้นทุนกับราคาที่กรอก แต่ถ้า
ต้นทุนยังกรอกไม่ครบ ตัวเลขนี้จะต่ำกว่าจริง
```

The wording is the owner's, not the implementer's. Every explanation lives in
**Explanations** below; a screen renders them and does not compose new ones.

**Empty state.** Every list and every screen has one, written as a sentence and
a single action. The empty state is a designed screen, not what is left when
content is missing. The demonstration can be reset without creating or clearing
a stored season.

**Locked banner.** A closed season shows a `bad`-bordered strip at the top of
every screen reading `ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้`, with
`ทำแผนฤดูถัดไปจากฤดูนี้` beside it. Inputs render as text, not as disabled
fields, because a greyed field invites tapping and a disabled control is the
worst way to say no.

**Bottom sheet.** For editing one line item or answering one question. Drag to
dismiss. Preferred over a full navigation for anything short.

**Tab bar.** Four items, fixed at the bottom, labels always visible.

## Screens

Navigation is a four-item bottom tab bar, the arrangement Shopee, LINE, and every
Thai banking application already taught this user.

```
[ หน้าแรก ]   [ กรอกข้อมูล ]   [ วิเคราะห์ ]   [ ฤดูกาล ]
```

Inside a season, the current item has a visible surface treatment and
`aria-current="page"`; focus is never conveyed by colour alone. The season-list
screen does not show three plan tabs that have no selected plan. If it was
opened from a season it shows one explicit route back; direct entry has no
invented back destination.

### Two explicit planning modes

A newly created season starts in `ประมาณการเร็ว`; an existing season remains in
`แผนละเอียด`. The active mode is always named under `โหมดที่ใช้อยู่`. Changing
mode is an explicit owner action (`เปลี่ยนเป็นแผนละเอียด` or
`ใช้ประมาณการเร็ว`), never an automatic conversion. Each mode keeps its own
inputs, so switching does not delete work.

Quick mode asks one question per page, in this fixed order:

1. `ฤดูกาลนี้คาดว่าจะขายทุเรียนได้กี่กิโลกรัม` with unit `กก.` and hint
   `ใช้ยอดที่คาดว่าจะขายได้จริงหลังหักผลเสียและผลที่ไม่ได้ขาย`
2. `คาดว่าจะขายได้ราคาเฉลี่ยกี่บาทต่อกิโลกรัม` with unit `บาท/กก.` and hint
   `ถ้ามีหลายเกรด ให้ใช้ราคาเฉลี่ยคร่าว ๆ ของทั้งฤดูกาล`
3. `คาดว่าฤดูกาลนี้มีต้นทุนรวมประมาณเท่าไร` with unit `บาท` and hint
   `รวมค่าใช้จ่ายทั้งหมดแบบคร่าว ๆ ก่อน รายละเอียดแยกทีหลังได้`

The number-field labels beneath those headings are deliberately shorter:
`ผลผลิตที่ขายได้โดยประมาณ`, `ราคาขายเฉลี่ยโดยประมาณ`, and
`ต้นทุนรวมโดยประมาณ`. They keep the question as the page title without
repeating the whole sentence inside the card.

Each page says `ขั้น 1 จาก 3`, `ขั้น 2 จาก 3`, or `ขั้น 3 จาก 3`, offers
`‹ ย้อนกลับ`, and can be left through a close target labelled
`พักและกลับหน้าฤดูกาล`. The primary actions are `ถัดไป` and, on the last page,
`ดูผลประมาณการ`. A saved quick-mode hub reports `1/3`, `2/3`, or `3/3` and uses
`ทำประมาณการต่อ` until it can use `ดูผลประมาณการ`.

The result begins with `ผลประมาณการ`, then `กำไรโดยประมาณ` or
`ขาดทุนโดยประมาณ` as the hero. Supporting rows are `รายได้โดยประมาณ`,
`ต้นทุนต่อกิโลกรัม`, and `ราคาขายคุ้มทุน`. A second card says
`คำนวณจาก 3 ค่านี้` and repeats `ผลผลิตที่ขายได้`, `ราคาขายเฉลี่ย`, and
`ต้นทุนรวม`. Its boundary is explicit:
`ผลนี้ใช้ประมาณการรวม ยังไม่ใช้เกรด รายการต้นทุน ROI ภาษี หรือคะแนนสุขภาพสวน`.
Actions are `แก้ประมาณการ` and `เปลี่ยนเป็นแผนละเอียด`.

Missing or invalid input never produces a partial answer. The result instead
says `ตอบให้ครบ 3 ข้อก่อนดูผล` and
`ระบบจะไม่เติมค่าที่ขาดหรือแสดงผลลัพธ์บางส่วนแทนข้อมูลจริง`, with the one action
`ทำประมาณการต่อ`. If a detailed route is opened while quick mode is active, it
says `ประมาณการเร็วกำลังใช้งาน` and
`ผลของฤดูกาลนี้คำนวณจากประมาณการเร็ว ระบบจึงไม่สลับไปใช้ข้อมูลละเอียดโดยอัตโนมัติ`.

### หน้าแรก — dashboard

Borrowed from a mobile banking home screen: the figure that matters is largest
and at the top, cards follow.

```
กำไรสุทธิ
834,600 บาท            ← hero
ฤดูกาล 2569 · สวน 10 ไร่

┌─────────────┬─────────────┐
│ ต้นทุน/กก.  │ จุดคุ้มทุน  │
│ 40.66 บาท   │ 4,670 กก.   │
├─────────────┼─────────────┤
│ ROI         │ คืนทุน      │
│ 119%        │ 0.78 ปี     │
└─────────────┴─────────────┘

ตอบสนองตลาด    79.8%   ⚠ ปรับปรุง
คะแนนสุขภาพ    3.0/5   ⚠ เฝ้าระวัง
```

When the plan is incomplete, the hero is replaced by what is missing and a
button that goes straight there. A dashboard must never show a confident figure
computed from absent inputs.

### กรอกข้อมูล — hub

Six cards, each showing its own completeness. Any order, any time, no wizard,
nothing lost by leaving. This screen is the workbook's `ตรวจสอบ` sheet made into
navigation.

```
ฤดู 2569                     ⋯
┌────────────────────────────┐
│ ✓ แผนตลาด               › │
│ ✓ ประมาณการผลผลิต       › │
│ ⚠ ปัจจัยและต้นทุน  3/16 › │
│ ○ ต้นทุนคงที่           › │
│ ○ เป้าหมาย         2/9  › │
│ ○ สุขภาพธุรกิจ     0/12 › │
└────────────────────────────┘

────────────────────────────
กำไรสุทธิโดยประมาณ  834,600.00 บาท
```

`เป้าหมาย` is a card because no target is a constant. Until the owner sets one,
the KPI that needs it has no verdict to give.

The hub edits year, name, and note while the season is open. Its management
area holds `ทำฤดูกาลถัดไปจากฤดูนี้` and `ปิดฤดูกาล`. Reset belongs only to the
browser demonstration; a real season is never mistaken for disposable sample
data.

### ประมาณการผลผลิต — grade mix

Grades are not a fixed set; a year may carry ten. The mix is an add-and-remove
list, each row carrying the owner's own name for the grade, its share, and its
price. A running total sits under the list in `good` when it reaches 100% and in
`bad` when it does not, so the error is visible while it is being made rather
than on save.

```
เกรด                สัดส่วน    ราคา/กก.
  A                   50%      100.00   ⋮
  B                   30%       80.00   ⋮
  C                   15%       50.00   ⋮
  ตกเกรด               5%       20.00   ⋮
  [ + เพิ่มเกรด ]
  ─────────────────────────────────────
  รวม                100%  ✓
  ราคาเฉลี่ยถ่วงน้ำหนัก      82.50 ฿/กก. ⓘ
```

### ปัจจัยและต้นทุน — line items

A cart. Rows of item, quantity, unit price, and line total, with the running
total pinned below and an add button. Tapping a row opens a bottom sheet.

Each row has a kind behind it, which is how the efficiency screen knows which
row is fertiliser and which is water. The owner sees the name, not the kind. Rows
they add themselves are `อื่นๆ` and feed no KPI, which the sheet says plainly.

Harvest labour, transport, and packing quantities follow the sellable yield by
definition, so they arrive filled in and marked `คำนวณจากผลผลิต` rather than
asked for twice. They stay editable, because a real orchard has reasons.

### สุขภาพธุรกิจ — twelve questions

One question per card, five full-width buttons from 1 to 5, thumb reachable.
Progress shown as `ข้อ 4 จาก 12`. Answers save as they are given.

### วิเคราะห์ — efficiency, checks, tax, scenario

Segmented control at the top: `ประสิทธิภาพ` · `ตรวจสอบ` · `ภาษี` · `สถานการณ์`.

**ประสิทธิภาพ** lists the nine KPIs as rows: actual, target, and the verdict
word. A KPI whose target the owner has not set shows `ยังไม่ได้ตั้งเป้า` and a
route to the targets card. It never invents a target and never grades a number
against one nobody chose.

**ตรวจสอบ** lists the six rules with a state and a button that jumps to the
input that would fix it.

**ภาษี** compares the two methods side by side and marks the cheaper one. The
workbook's own disclaimer, that this is an estimate and not tax advice, sits on
this screen where the figures are, at `caption`. It is not moved elsewhere.

**สถานการณ์** does not begin as a grid. Two sliders, `ราคา` and `ผลผลิต`, and one
large figure showing the profit that results. A 5×5 table is unreadable on a
phone and lets the owner do less; sliders let them ask their actual question,
which is how far things can fall before this stops working. `ดูตารางทั้งหมด`
opens the full matrix as a horizontally scrolling table with the first column
pinned, for anyone who wants the overview.

### ฤดูกาล

Each saved season is the combined business forecast for every orchard plot in
one Buddhist harvest year. A season has a required four-digit year, a required
human name, and one optional note. One owner has one season per year. The year
is stored separately and is never inferred from the name.

Cards state both time and lifecycle: the highest year says `ปีล่าสุด`; an open
season says `กำลังทำ`; an older open season says `ปีก่อน · ยังไม่ปิด`; a closed
season says `ปิดแล้ว`. Text carries each fact and colour only reinforces it. A
note is previewed in at most two lines. Metadata is editable while open and
read-only after close.

The workbook sample is not a season. `ดูตัวอย่างการใช้งาน` opens an editable,
resettable browser-only demonstration with `สร้างฤดูกาลของฉัน`; leaving or
reopening it writes no history. After a real season exists, the demonstration
remains available as a quiet help action outside the season list.

### Sign-in, registration, password reset

Three plain screens. One field per row, one primary button, and the reset link
under the password field. No illustration, no marketing copy, no social buttons.

## User flow

### Entering

```
เปิดแอป
  │
  ├─ ยังไม่ล็อกอิน ─→ เข้าสู่ระบบ ─┬─ [ลืมรหัสผ่าน] → กรอกอีเมล → ลิงก์ในเมล
  │                                │        → ตั้งรหัสใหม่ → session เดิมถูกล้างทั้งหมด
  │                                └─ [สมัครใหม่] → อีเมล + รหัสผ่าน → เปิด Mailpit
  │                                                        → กดลิงก์ยืนยัน → เข้าสู่ระบบ
  │
  └─ ล็อกอินแล้ว ─→ มีแผนไหม?
                      ├─ ไม่มี → ┬─ [ดูตัวอย่างการใช้งาน] — ไม่บันทึก
                      │          └─ [สร้างฤดูกาลแรก] — ปี + ชื่อ + บันทึก
                      └─ มี   → หน้าแรก
```

### First run, from nothing to a profit figure

```
เลือก [ดูตัวอย่างการใช้งาน]
        ↓
   เห็นผลจากไฟล์ต้นฉบับ ลองเปลี่ยนตัวเลข และคืนค่าได้โดยไม่บันทึก
        ↓
   [ สร้างฤดูกาลของฉัน ] → ปี พ.ศ. + ชื่อ + บันทึก
        ↓
   กรอกข้อมูล (hub) — ทั้ง 6 การ์ดว่าง   แถบล่าง: "ยังคำนวณกำไรสุทธิไม่ได้"
        ↓
   ประมาณการผลผลิต + เกรด               แถบล่าง: รายได้ 1,645,875.00 ฿
        ↓
   ปัจจัยและต้นทุน                       แถบล่างขยับทุกตัวอักษรที่พิมพ์
     (เก็บเกี่ยว/ขนส่ง/คัดแยก เติมมาให้แล้ว)
        ↓
   ต้นทุนคงที่                           แถบล่าง: กำไรสุทธิ 834,600.00 ฿
        ↓
   เป้าหมาย  ← ถ้าข้ามไป KPI จะบอกว่า "ยังไม่ได้ตั้งเป้า" ไม่ใช่ตัดสินมั่ว
        ↓
   สุขภาพธุรกิจ 12 ข้อ
        ↓
   ✓ ครบ → หน้าแรกเปิดใช้ได้เต็ม
```

Nothing above is a required order and nothing is lost by leaving. The only real
dependency is that yield comes before anything can be computed at all.

### Day to day

```
เปิดแอป → หน้าแรก
   กำไรสุทธิ 834,600.00 บาท  ⓘ
   ต้นทุน/กก. 40.66  ⓘ   จุดคุ้มทุน 4,670.00 กก.  ⓘ
   ROI 119.23%  ⓘ         คืนทุน 0.78 ปี  ⓘ
   ตอบสนองตลาด 79.80%   ⚠ ปรับปรุง      ←── แตะ
                               ↓
                    วิเคราะห์ › ประสิทธิภาพ
                    ผลผลิตต่อไร่  1,995.00 / เป้า 2,200.00  ⚠
                    ผลผลิตต่อ กก.ปุ๋ย  3.99 / ยังไม่ได้ตั้งเป้า →
                               ↓ แตะแถวใดก็ได้
                    กระโดดไปช่องที่แก้มันได้
```

### The decision this application exists for

```
ล้งโทรมาเสนอ 75 บาท/กก. (ตั้งไว้ 82.50)
        ↓
วิเคราะห์ › สถานการณ์
   ราคา    ─────●──────────  −10%
   ผลผลิต  ──────────●─────    0%
        ↓
   กำไรสุทธิ  669,013.00 บาท
   เหนือจุดคุ้มทุน 15,280.00 กก.
        ↓
   ตอบได้ขณะยังถือสายอยู่
```

### Closing a season, and starting the next

```
กรอกข้อมูล › ⋯ › ปิดฤดูกาล
        ↓
  ยืนยัน → ฤดู 2569 กลายเป็นอ่านอย่างเดียว
        ↓
  ทุกหน้าขึ้นแถบ: ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้
                  [ ทำแผนฤดูถัดไปจากฤดูนี้ ]
        ↓
  แตะ → ฟอร์มเสนอปี 2570 ชื่อเดิม และบันทึกว่างให้ตรวจก่อนสร้าง
        → ฤดู 2570 ถูกสร้างจากสำเนาข้อมูลคำนวณทั้งหมดของ 2569
        ↓
  แก้เฉพาะที่เปลี่ยน (ราคา ผลผลิต ค่าแรง)
  ไม่ต้องกรอก 16 บรรทัดใหม่
```

Closing and duplicating are one motion on purpose. The moment an owner accepts
that a season is over is the moment the next one is worth starting.

## Explanations

Every figure below carries a `ⓘ`. Each answers the same four questions in the
same order: `คืออะไร`, `ใช้ยังไง`, `ทำไมต้องมี`, `ไม่ใส่ได้ไหม`.

**The wording is the owner's.** The drafts below exist so there is something to
correct rather than a blank, and they come from reading the workbook, not from
attending the training the workbook came from. Correct them freely.

No explanation names a number that counts as good. Where a figure is graded,
the grade comes from a target the owner sets, never from a benchmark this
document asserts.

### กระแสเงินสด กับ กำไรสุทธิ ต่างกันยังไง

The most misread pair in the whole workbook, so it gets one shared explanation
reachable from both.

- **คืออะไร** — กำไรสุทธิรวมค่าเสื่อมราคาซึ่งไม่ได้จ่ายเป็นเงินสดจริงในปีนี้
  กระแสเงินสดตัดค่าเสื่อมออก จึงเป็นเงินที่เข้ากระเป๋าจริง
- **ใช้ยังไง** — ใช้กำไรสุทธิดูว่าธุรกิจกำไรไหม ใช้กระแสเงินสดดูว่าเดือนหน้ามี
  เงินจ่ายค่าแรงหรือเปล่า
- **ทำไมต้องมี** — สวนที่กำไรดีแต่เงินสดขาดมือ ล้มได้ และล้มบ่อย
- **ไม่ใส่ได้ไหม** — คำนวณให้เอง แต่ถ้าไม่แยกว่าต้นทุนคงที่ตัวไหนเป็นเงินสด
  ตัวเลขนี้จะเท่ากับกำไรสุทธิ และจะไม่บอกอะไรเลย

### จุดคุ้มทุน

- **คืออะไร** — ปริมาณที่ต้องขายให้ได้ก่อนจะเริ่มมีกำไร
- **ใช้ยังไง** — เทียบกับผลผลิตที่คาดว่าจะได้ ยิ่งห่างยิ่งมีที่ให้พลาด
- **ทำไมต้องมี** — บอกว่าราคาตกได้แค่ไหนก่อนขาดทุน ใช้ตอนล้งเสนอราคาได้ทันที
- **ไม่ใส่ได้ไหม** — คำนวณให้เอง แต่ถ้าต้นทุนกรอกไม่ครบ ตัวเลขจะต่ำกว่าความจริง

### ส่วนเผื่อความปลอดภัย

- **คืออะไร** — ผลผลิตที่เกินจุดคุ้มทุนอยู่กี่กิโลกรัม
- **ใช้ยังไง** — ยิ่งมากยิ่งทนต่อปีที่อากาศไม่เป็นใจ
- **ทำไมต้องมี** — เป็นตัวเดียวที่บอกว่ารับความเสี่ยงได้แค่ไหนเป็นตัวเลข
- **ไม่ใส่ได้ไหม** — มาจากจุดคุ้มทุน ไม่ต้องกรอกเพิ่ม

### ต้นทุนคงที่ เงินสด กับ ไม่ใช่เงินสด

- **คืออะไร** — ค่าเสื่อมระบบน้ำ ค่าเสื่อมรถ เป็นต้นทุนที่ลงบัญชีแต่ปีนี้ไม่ได้
  ควักเงินจ่าย ส่วนค่าเช่า ดอกเบี้ย ค่าแรงประจำ จ่ายจริงทุกปี
- **ใช้ยังไง** — ติ๊กให้ถูกประเภทตอนกรอก
- **ทำไมต้องมี** — เป็นสิ่งเดียวที่ทำให้กระแสเงินสดกับกำไรสุทธิต่างกันได้
- **ไม่ใส่ได้ไหม** — ใส่ผิดประเภทได้ แต่กระแสเงินสดจะผิดตาม

### ราคาขายเฉลี่ยถ่วงน้ำหนัก

- **คืออะไร** — ถ้าเทผลผลิตทั้งสวนรวมเป็นกองเดียวแล้วขายหมดกองในราคาเดียว
  ราคานั้นคือตัวนี้ ไม่ใช่ราคาของเกรดใดเกรดหนึ่ง และไม่ใช่ราคาทุกเกรดบวกกัน
  หารจำนวนเกรด เกรดที่มีของมากดึงราคาเข้าหาตัวมันมากกว่า
- **ใช้ยังไง** — คูณกับผลผลิตที่ขายได้ ก็คือรายได้ทั้งฤดู ถ้าอยากให้ราคานี้ขึ้น
  โดยไม่ต้องขอขึ้นราคาจากผู้ซื้อ ทำได้ทางเดียวคือดันของจากเกรดล่างขึ้นเกรดบน
- **ทำไมต้องมี** — เวลาคุยราคา เราคุยกันทีละเกรด แต่เงินที่เข้ากระเป๋าจริงมาจาก
  ส่วนผสมของทุกเกรดรวมกัน ตัวนี้คือส่วนผสมนั้นย่อเหลือตัวเลขเดียว
- **ไม่ใส่ได้ไหม** — ไม่ต้องกรอก คำนวณจากสัดส่วนเกรดคูณราคาแต่ละเกรด แต่ถ้า
  สัดส่วนเกรดยังรวมไม่ครบ ตัวนี้จะไม่ขึ้นเลย เพราะยังไม่รู้ว่าของส่วนที่หายไป
  เป็นเกรดอะไร

### ต้นทุนผันแปร กับ ต้นทุนคงที่

- **คืออะไร** — ต้นทุนผันแปรคือของที่ใช้มากขึ้นเมื่อได้ผลผลิตมากขึ้น ปุ๋ย ยา น้ำ
  ไฟ น้ำมัน ค่าเก็บ ค่าขนส่ง ค่าบรรจุ ต้นทุนคงที่คือของที่จ่ายเท่าเดิมไม่ว่าปีนั้น
  จะได้กี่กิโล ค่าเช่าที่ ดอกเบี้ย ค่าแรงประจำ ค่าเสื่อมระบบน้ำ
- **ใช้ยังไง** — ตอนกรอก ถามตัวเองข้อเดียว ถ้าปีนี้ไม่ได้ผลผลิตเลย ยังต้องจ่าย
  ตัวนี้ไหม ถ้ายังต้องจ่าย มันคือต้นทุนคงที่
- **ทำไมต้องมี** — จุดคุ้มทุนเอาต้นทุนคงที่ทั้งก้อนมาเป็นตัวตั้ง แล้วหารด้วยเงิน
  ที่เหลือจากทุเรียนหนึ่งกิโลหลังหักต้นทุนผันแปรออกแล้ว สองก้อนนี้ทำหน้าที่คนละ
  อย่างในสูตรเดียวกัน วางสลับที่เมื่อไร จุดคุ้มทุนผิดทันที
- **ไม่ใส่ได้ไหม** — ต้องแยกเอง เครื่องแยกให้ไม่ได้ ค่าปุ๋ยที่ไปนั่งอยู่ในช่อง
  ต้นทุนคงที่จะทำให้จุดคุ้มทุนสูงกว่าความจริง และตัวเลขจะไม่ฟ้องว่าผิด เพราะยอด
  รวมยังเท่าเดิมทุกบาท

### ส่วนเกินต่อหน่วย

- **คืออะไร** — ทุเรียนหนึ่งกิโลขายได้เท่าไร หักค่าของที่ต้องใส่ลงไปเพื่อให้ได้
  กิโลนั้นออก เหลือเท่าไรคือตัวนี้ เงินที่เหลือยังไม่ใช่กำไร มันคือเงินที่เอาไป
  ทยอยจ่ายค่าเช่า ดอกเบี้ย และค่าเสื่อม
- **ใช้ยังไง** — ทุกกิโลที่ขายได้ ให้นึกว่าหย่อนเงินก้อนนี้ลงในถังต้นทุนคงที่
  วันที่ถังเต็มคือจุดคุ้มทุน กิโลถัดจากนั้นเป็นกำไรเต็ม ๆ
- **ทำไมต้องมี** — ถ้าตัวนี้ติดลบ ยิ่งขายยิ่งขาดทุน และแก้ด้วยการขายให้มากขึ้น
  ไม่ได้เลย เป็นสัญญาณเดียวที่บอกได้ตั้งแต่ก่อนลงมือว่าปีนี้ไม่ควรทำ
- **ไม่ใส่ได้ไหม** — ไม่ต้องกรอก มาจากราคาเฉลี่ยถ่วงน้ำหนักลบต้นทุนผันแปรต่อกิโล
  แต่ถ้าต้นทุนผันแปรกรอกไม่ครบ ตัวนี้จะดูดีเกินจริง และจุดคุ้มทุนจะดูใกล้เกินจริง
  ตามไปด้วย

### ตอบสนองตลาด

- **คืออะไร** — ของที่เรามีขาย เทียบกับของที่ตลาดต้องการ ได้ 79.8% แปลว่าเขา
  อยากได้ร้อย เรามีให้แปดสิบ
- **ใช้ยังไง** — ต่ำกว่าร้อยคือมีคำสั่งซื้อที่รับไม่ไหว เกินร้อยคือมีของที่ยังไม่มี
  คนรับ สองทางนี้เสียเงินคนละแบบ และแก้คนละวิธี
- **ทำไมต้องมี** — เป็นตัวเดียวที่เอาสองหน้าซึ่งกรอกแยกกันมาชนกัน ความต้องการ
  ของตลาดที่กรอกในหน้าตลาด กับผลผลิตที่ประมาณในหน้าผลผลิต ถ้าไม่มีตัวนี้ สองหน้า
  นั้นจะโตไปคนละทางโดยไม่มีอะไรมาทัก
- **ไม่ใส่ได้ไหม** — ต้องกรอกความต้องการของตลาดก่อน ถ้าไม่กรอกหรือกรอกเป็นศูนย์
  ตัวนี้จะไม่ขึ้น

### สัดส่วนลูกค้ารายใหญ่ที่สุด

- **คืออะไร** — ของทั้งสวนไปอยู่ในมือผู้ซื้อรายใหญ่ที่สุดกี่เปอร์เซ็นต์ ตัวนี้กรอก
  เอง ไม่ได้คำนวณให้
- **ใช้ยังไง** — ตอบคำถามเดียว ถ้าเจ้านี้ไม่มาปีหน้า เราเหลืออะไร
- **ทำไมต้องมี** — เป็นความเสี่ยงชนิดที่ไม่โผล่ในกำไรเลย ปีที่ขายเจ้าเดียวทั้งสวน
  กับปีที่กระจายสามเจ้า ให้กำไรเท่ากันเป๊ะได้ แต่แบบแรกอยู่ห่างจากศูนย์แค่โทรศัพท์
  สายเดียว
- **ไม่ใส่ได้ไหม** — ได้ และตัวเลขอื่นจะไม่เพี้ยน เพราะตัวนี้ไม่ได้เข้าสูตรไหนเลย
  แต่การเว้นว่างคือการเลือกที่จะไม่มองความเสี่ยงข้อนี้เป็นตัวเลข

### ROI

- **คืออะไร** — กำไรทั้งฤดู เทียบกับเงินก้อนที่จมอยู่กับสวน ระบบน้ำ รถ โรงเรือน
  และของลงทุนอื่นที่ลงไปแล้ว ได้ 119% แปลว่าเงินที่จมอยู่หนึ่งบาท ปีนี้ทำกำไรกลับ
  มาหนึ่งบาทสิบเก้าสตางค์
- **ใช้ยังไง** — ใช้เทียบสวนกับทางเลือกอื่นของเงินก้อนเดียวกัน ไม่ใช่เทียบกับสวน
  ข้างบ้าน เพราะเงินลงทุนตั้งต้นของแต่ละสวนไม่เหมือนกันเลย
- **ทำไมต้องมี** — กำไรบอกว่าปีนี้ได้เท่าไร ROI บอกว่าคุ้มกับที่ทุ่มลงไปไหม สวนที่
  กำไรหกแสนจากเงินลงทุนสิบล้าน แพ้สวนที่กำไรสามแสนจากเงินลงทุนห้าแสน
- **ไม่ใส่ได้ไหม** — ไม่ต้องกรอกเงินลงทุนให้ทุกรายการ ค่าแรงประจำ ค่าเช่า และ
  ค่าใช้จ่ายรายปีที่ไม่มีเงินก้อนเริ่มต้นเว้นได้ แต่ต้องมีเงินลงทุนมากกว่าศูนย์
  อย่างน้อยหนึ่งรายการจึงจะคำนวณ ROI ได้ ระบบจะไม่เดาเงินลงทุนให้

### ระยะคืนทุน

- **คืออะไร** — ถ้าปีต่อ ๆ ไปเงินสดเข้ามาเท่าปีนี้ ต้องใช้กี่ปีเงินก้อนที่ลงไปตอน
  แรกถึงจะกลับมาครบ
- **ใช้ยังไง** — เทียบกับอายุของสิ่งที่ลงทุนไป ถ้าระบบน้ำอยู่ได้สิบปีแต่คืนทุนสิบ
  สองปี แปลว่ามันพังก่อนจะคุ้ม ตัวเลขนี้สมมติว่าทุกปีเหมือนปีนี้ ซึ่งไม่จริง แต่หยาบ
  พอจะบอกได้ว่าเรากำลังพูดถึงสามปีหรือสิบห้าปี
- **ทำไมต้องมี** — เป็นตัวเดียวในหน้านี้ที่พูดเป็นหน่วยเวลา ไม่ใช่หน่วยเงิน คำถาม
  ว่าอีกกี่ปี ตอบตัวเองได้ง่ายกว่าคำถามว่ากี่เปอร์เซ็นต์
- **ไม่ใส่ได้ไหม** — ต้องมีเงินลงทุนมากกว่าศูนย์อย่างน้อยหนึ่งรายการ และกระแส
  เงินสดต้องเป็นบวก ถ้ายังไม่มีเงินลงทุน หรือกระแสเงินสดเป็นศูนย์หรือติดลบ ระบบจะ
  บอกเหตุผลแทนการแสดงจำนวนปีที่ทำให้เข้าใจผิด

### คะแนนสุขภาพธุรกิจ

- **คืออะไร** — คำถาม 12 ข้อ ให้คะแนนตัวเองข้อละ 1 ถึง 5 แล้วเฉลี่ย จัดเป็น 6 ด้าน
  ด้านละ 2 ข้อ การเงิน ผลผลิต ตลาด ทรัพยากร คน และความทนทาน
- **ใช้ยังไง** — อย่าดูแต่คะแนนรวม ให้ดูว่าด้านไหนต่ำสุด คะแนนรวม 3.0 ที่มาจากทุก
  ด้านได้ 3 เท่ากัน ไม่เหมือนคะแนนรวม 3.0 ที่มีด้านหนึ่งได้ 1 แล้วด้านอื่นดึงขึ้นมา
- **ทำไมต้องมี** — ตัวเลขอื่นทั้งแอปคำนวณจากเงิน ส่วนนี้เป็นที่เดียวที่บันทึกสิ่งที่
  เงินมองไม่เห็น แรงงานพอไหม ราคาผันผวนแล้วรับไหวไหม มีแผนสำรองหรือเปล่า
- **ไม่ใส่ได้ไหม** — ตอบไม่ครบทั้ง 12 ข้อ คะแนนรวมจะไม่ขึ้นเลย ไม่ใช่เฉลี่ยเฉพาะ
  ข้อที่ตอบ และด้านใดที่ยังตอบไม่ครบสองข้อ ด้านนั้นก็จะยังไม่มีคะแนน ที่เป็นแบบนี้
  เพราะคะแนนจากการตอบครึ่งเดียว ไม่ได้แปลว่าสวนแข็งแรง มันแปลว่าเรายังไม่ได้ถาม
  ตัวเองให้ครบ

### ภาษี สองวิธี

- **คืออะไร** — ในการคำนวณภาษี เราหักค่าใช้จ่ายออกจากรายได้ก่อน แล้วค่อยคิดภาษีจาก
  ส่วนที่เหลือ วิธีแรกหักตามที่จ่ายจริง ซึ่งก็คือต้นทุนทั้งหมดที่กรอกไว้ในแอปนี้แล้ว
  วิธีที่สองไม่สนว่าจ่ายจริงเท่าไร หักเหมาเป็นสัดส่วนคงที่ของรายได้ จากนั้นทั้งสอง
  วิธีหักค่าลดหย่อนส่วนตัวเท่ากัน แล้วคิดภาษีตามขั้นบันไดชุดเดียวกัน
- **ใช้ยังไง** — ดูว่าวิธีไหนเสียน้อยกว่า แอปทำเครื่องหมายให้ หลักคิดคือถ้าต้นทุน
  จริงสูงกว่าอัตราหักเหมา หักตามจริงคุ้มกว่า ถ้าต่ำกว่า หักเหมาคุ้มกว่า แลกกับว่า
  หักตามจริงต้องมีหลักฐานการจ่ายเก็บไว้ ส่วนหักเหมาไม่ต้อง
- **ทำไมต้องมี** — สองวิธีนี้ให้ผลต่างกันได้มาก จากตัวเลขชุดเดียวกันเป๊ะ โดยไม่ต้อง
  ทำอะไรต่างกันเลยตลอดทั้งปี รู้ล่วงหน้าตั้งแต่ตอนวางแผน ดีกว่ามารู้ตอนยื่น
- **ไม่ใส่ได้ไหม** — ไม่ต้องกรอกเพิ่ม คำนวณจากรายได้และต้นทุนที่กรอกไว้แล้ว แต่ตัวเลข
  นี้เป็นการประมาณจากข้อมูลในแอปเท่านั้น ไม่ได้รวมรายได้ทางอื่นและค่าลดหย่อนอื่นที่
  คุณมี อัตราที่ใช้มาจากแบบคำนวณต้นฉบับและยังไม่ได้ตรวจสอบว่าตรงกับกฎหมายปัจจุบัน
  นี่ไม่ใช่คำแนะนำทางภาษี ก่อนยื่นจริงให้ยืนยันกับผู้ที่ดูแลภาษีของคุณ

### เป้าหมาย KPI

ทั้งเก้าตัวใช้กติกาเดียวกัน แอปแสดงค่าจริงที่คำนวณได้เสมอ แต่จะไม่ตัดสินว่าผ่านหรือ
ไม่ผ่าน จนกว่าคุณจะตั้งเป้าของตัวเองในหน้าเป้าหมาย ที่ยังไม่ตั้งจะขึ้นว่า
`ยังไม่ได้ตั้งเป้า` พร้อมทางลัดไปตั้ง แอปจะไม่ตั้งเป้าให้ เพราะเป้าที่เหมาะกับสวน
หนึ่ง ใช้กับอีกสวนไม่ได้ และเป้าที่เครื่องตั้งให้ ไม่มีใครรู้สึกว่าเป็นของตัวเอง

เจ็ดตัวแรกยิ่งสูงยิ่งดี สองตัวสุดท้าย สัดส่วนสูญเสียกับต้นทุนต่อกิโลกรัม ยิ่งต่ำ
ยิ่งดี แอปรู้ว่าตัวไหนกลับด้าน ไม่ต้องกรอกบอก

#### ผลผลิตต่อไร่

- **คืออะไร** — ผลผลิตที่ขายได้ทั้งสวน หารด้วยจำนวนไร่
- **ใช้ยังไง** — เทียบสวนตัวเองข้ามปี ปีนี้กับปีที่แล้วบนที่ดินผืนเดิม
- **ทำไมต้องมี** — ผลผลิตรวมโตขึ้นเพราะขยายพื้นที่ก็ได้ ต่อไร่คือตัวที่แยกว่าเรา
  ทำได้ดีขึ้นจริง หรือแค่ใหญ่ขึ้น
- **ไม่ใส่ได้ไหม** — ต้องกรอกพื้นที่เป็นไร่ ถ้าไม่กรอก ตัวนี้จะไม่ขึ้น

#### ผลผลิตต่อต้น

- **คืออะไร** — ผลผลิตที่ขายได้ หารด้วยจำนวนต้นที่ให้ผลแล้ว ต้นที่ยังไม่ให้ผลไม่นับ
- **ใช้ยังไง** — ดูว่าต้นทำงานได้เต็มที่ไหม แยกออกจากเรื่องปลูกถี่หรือปลูกห่าง
- **ทำไมต้องมี** — ผลผลิตต่อไร่ดีขึ้นได้ด้วยการปลูกถี่ขึ้น ซึ่งไม่ได้แปลว่าต้น
  แข็งแรงขึ้น ต่อต้นเป็นตัวที่แยกสองเรื่องนี้ออกจากกัน
- **ไม่ใส่ได้ไหม** — ใช้จำนวนต้นที่ให้ผล ซึ่งเป็นตัวเดียวกับที่ใช้ประมาณผลผลิตอยู่
  แล้ว ไม่ต้องกรอกเพิ่ม

#### ผลผลิตต่อวันแรงงาน

- **คืออะไร** — ผลผลิตที่ขายได้ หารด้วยจำนวนวันแรงงานดูแลสวนที่กรอกไว้ในต้นทุน
  ผันแปร
- **ใช้ยังไง** — ขยับได้สองทาง ได้ของมากขึ้นด้วยแรงเท่าเดิม หรือได้ของเท่าเดิมด้วย
  แรงน้อยลง
- **ทำไมต้องมี** — แรงงานเป็นต้นทุนก้อนใหญ่ที่สุดของหลายสวน และเป็นก้อนที่หาได้ยาก
  ขึ้นทุกปี
- **ไม่ใส่ได้ไหม** — ต้องกรอกจำนวนวันแรงงาน ไม่ใช่แค่ยอดเงิน กรอกแต่เงิน ตัวนี้จะ
  ไม่ขึ้น

#### ผลผลิตต่อปุ๋ย

- **คืออะไร** — ได้ทุเรียนกี่กิโลกรัม ต่อปุ๋ยที่ใส่ลงไปหนึ่งกิโลกรัม
- **ใช้ยังไง** — ใส่ปุ๋ยเพิ่มแล้วตัวนี้ตกลง แปลว่าปุ๋ยส่วนที่เพิ่มไม่ได้กลายเป็นผล
- **ทำไมต้องมี** — ปุ๋ยเป็นต้นทุนผันแปรที่ใหญ่ที่สุดในหลายสวน และเป็นตัวที่ปรับได้
  เร็วที่สุดในฤดูถัดไป
- **ไม่ใส่ได้ไหม** — ต้องกรอกปริมาณปุ๋ยเป็นกิโลกรัม ไม่ใช่แค่ยอดเงิน

#### ผลผลิตต่อน้ำ

- **คืออะไร** — ได้ทุเรียนกี่กิโลกรัม ต่อน้ำหนึ่งลูกบาศก์เมตร
- **ใช้ยังไง** — เทียบข้ามปี โดยเฉพาะปีที่ฝนต่างกันมาก
- **ทำไมต้องมี** — น้ำเป็นทรัพยากรที่ปีไหนขาดก็ซื้อเพิ่มไม่ได้ รู้ว่าตัวเองใช้คุ้ม
  แค่ไหน คือรู้ว่าปีแล้งจะเหลือที่ให้ขยับเท่าไร
- **ไม่ใส่ได้ไหม** — ต้องกรอกปริมาณน้ำเป็นลูกบาศก์เมตร

#### ผลผลิตต่อไฟฟ้า

- **คืออะไร** — ได้ทุเรียนกี่กิโลกรัม ต่อไฟฟ้าหนึ่งหน่วย
- **ใช้ยังไง** — ส่วนใหญ่คือค่าสูบน้ำ ตัวนี้ตกโดยที่ผลผลิตไม่เพิ่ม มักแปลว่ามีอะไร
  รั่วหรือปั๊มทำงานหนักกว่าที่ควร
- **ทำไมต้องมี** — เป็นบิลที่มาทุกเดือนจนคุ้นจนไม่ได้มอง การผูกมันกับผลผลิตทำให้
  มันกลับมาถูกมองอีกครั้ง
- **ไม่ใส่ได้ไหม** — ต้องกรอกจำนวนหน่วยไฟ ไม่ใช่แค่ยอดเงินตามบิล

#### สัดส่วนเกรดคุณภาพ

- **คืออะไร** — สัดส่วนของเกรดที่คุณติ๊กไว้ว่านับเป็นเกรดคุณภาพ รวมกันได้เท่าไร
- **ใช้ยังไง** — ตัวนี้กับราคาเฉลี่ยถ่วงน้ำหนักขยับไปด้วยกันเสมอ ดันตัวนี้ขึ้นได้
  ราคาเฉลี่ยขึ้นตามโดยไม่ต้องเจรจาราคาใหม่
- **ทำไมต้องมี** — เป็นเป้าหมายเดียวในเก้าตัวที่พูดถึงคุณภาพ ไม่ใช่ปริมาณ
- **ไม่ใส่ได้ไหม** — ต้องติ๊กว่าเกรดไหนนับเป็นเกรดคุณภาพ ถ้าไม่ติ๊กเลย ตัวนี้จะเป็น
  ศูนย์ ซึ่งไม่ได้แปลว่าของไม่ดี แปลว่ายังไม่ได้บอกแอปว่าเกรดไหนคือของดี

#### สัดส่วนสูญเสีย

- **คืออะไร** — ของที่หายไประหว่างทางก่อนถึงมือผู้ซื้อ คิดเป็นกี่เปอร์เซ็นต์ของ
  ผลผลิตทั้งหมด
- **ใช้ยังไง** — ตัวนี้ยิ่งต่ำยิ่งดี และมันไม่ได้กระทบแค่ตัวมันเอง เพราะผลผลิตที่
  ขายได้ซึ่งเป็นตัวตั้งของ KPI แทบทุกตัว คำนวณหลังหักส่วนนี้ออกแล้ว
- **ทำไมต้องมี** — ของที่เสียไป จ่ายค่าปุ๋ยค่าน้ำค่าแรงไปครบเหมือนของที่ขายได้ทุก
  บาท ต่างกันแค่ไม่มีรายได้กลับมา
- **ไม่ใส่ได้ไหม** — เป็นตัวที่กรอกเอง และถ้าไม่กรอก ผลผลิตที่ขายได้จะไม่ขึ้น ซึ่ง
  ทำให้เกือบทั้งหน้าวิเคราะห์ว่างตามไปด้วย

#### ต้นทุนต่อกิโลกรัม

- **คืออะไร** — ต้นทุนทั้งหมด ทั้งผันแปรและคงที่ หารด้วยผลผลิตที่ขายได้ คือต้นทุน
  จริงของทุเรียนหนึ่งกิโลที่ออกจากสวนนี้
- **ใช้ยังไง** — เทียบกับราคาที่ผู้ซื้อเสนอมาได้ทันที สูงกว่าคือมีกำไร ต่ำกว่าคือ
  ขาดทุนตั้งแต่ตกลง
- **ทำไมต้องมี** — เป็นตัวเลขเดียวในแอปที่เอาไปใช้ได้ทันทีขณะยืนคุยราคาอยู่หน้าสวน
- **ไม่ใส่ได้ไหม** — ต้องกรอกต้นทุนให้ครบทั้งสองก้อน ยิ่งกรอกไม่ครบ ตัวนี้ยิ่งดูดี
  เกินจริง และมันยิ่งต่ำยิ่งดี จึงเป็นตัวที่หลอกตัวเองได้ง่ายที่สุดในเก้าตัว

## Rules that are not negotiable

1. Every interactive target is at least 48 by 48.
2. Every text colour clears 6:1 against the surface behind it.
3. No good or bad state is signalled by colour alone.
4. `prefers-reduced-motion: reduce` removes all motion.
5. Every input is reachable and operable with one thumb.
6. The tax disclaimer appears on the tax screen itself.
7. Nothing user-facing is set below 13, and Thai line height never drops below
   1.55.
8. No screen shows a verdict against a target the owner has not set, and no
   screen shows a confident figure derived from inputs that are missing.

## Open questions

- Two decimal places on the dashboard hero costs width on a narrow phone:
  `834,600.00 บาท` is eleven characters before the unit. The rule is the owner's
  and stands; this is noted only so it can be revisited after it is seen on a
  real device rather than argued about now.
- Whether an offline mode is wanted. Orchard signal is not assumed to be good,
  and the calculation already runs locally, so the gap is only saving. Not
  planned; raised because the context suggests it.
- Whether one owner ever compares two seasons side by side, which would change
  the dashboard from a single plan to a comparison. Closing a season and
  duplicating it makes this more likely, not less.
- Icon set is not chosen.

Resolved since revision 0.1: the grade mix is an add-and-remove list rather than
a split control, because a year may carry ten grades.

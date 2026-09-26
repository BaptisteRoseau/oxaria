# WCAG 2.2 Accessibility Checker Rules

This document lists automatable/semi-automatable accessibility rules derived from the [WCAG 2.2](https://www.w3.org/TR/WCAG22/) success criteria and their associated [Techniques](https://www.w3.org/WAI/WCAG22/Techniques/) and [Failures](https://www.w3.org/WAI/WCAG22/Techniques/failures/) as published in this repository. Each rule maps to a WCAG Technique/Failure ID where one exists in the repo; where no published WCAG 2.2 technique or failure covers the rule (a success criterion new in WCAG 2.2, a best practice stated only in an Understanding document, or a failure made obsolete in WCAG 2.2), a custom three-letter category code is used instead (e.g. `TGT001`).

Rules are grouped by category. Each entry cites the WCAG Success Criterion(s) (SC) it supports.

## Table of Contents

1. [Text Alternatives (Images)](#text-alternatives-images)
2. [Forms & Labels](#forms--labels)
3. [Headings & Document Structure](#headings--document-structure)
4. [Language](#language)
5. [Links](#links)
6. [Color & Contrast](#color--contrast)
7. [Tables](#tables)
8. [ARIA Usage](#aria-usage)
9. [Multimedia](#multimedia)
10. [Keyboard & Focus](#keyboard--focus)
11. [Navigation](#navigation)
12. [Timing & Motion](#timing--motion)
13. [Pointer & Target Size (WCAG 2.2)](#pointer--target-size-wcag-22)
14. [Cognitive & Authentication (WCAG 2.2)](#cognitive--authentication-wcag-22)
15. [Resize & Reflow](#resize--reflow)

---

## Text Alternatives (Images)

### H37 - Provide alt text on informative images

Every `img` that conveys meaning must have an `alt` attribute whose text serves the same purpose as the image. (SC 1.1.1)

#### DON'T

```html
<img src="newsletter-banner.gif">
```

#### DO

```html
<img src="newsletter-banner.gif" alt="Free newsletter. Get free recipes, news, and more. Learn more.">
```

---

### H67 - Use empty alt (and no title) for purely decorative images

Images that add no information (borders, spacers, background flourishes) must have `alt=""` and no `title` attribute so assistive technology skips them instead of announcing noise. (SC 1.1.1)

#### DON'T

```html
<img src="decorative-swirl.png" alt="decorative swirl" title="swirl graphic">
```

#### DO

```html
<img src="decorative-swirl.png" alt="">
```

---

### H36 - Provide alt text on image submit buttons

`input type="image"` elements must have `alt` text describing the action the button performs, since the image itself is the only visible label. (SC 1.1.1)

#### DON'T

```html
<input type="image" src="submit-btn.png">
```

#### DO

```html
<input type="image" src="submit-btn.png" alt="Submit search">
```

---

### H24 - Provide text alternatives for image map areas

Each `area` element inside a `map` used for an image map must carry its own `alt` text describing where that region links to. (SC 1.1.1)

#### DON'T

```html
<map name="floorplan">
  <area shape="rect" coords="0,0,100,100" href="kitchen.html">
  <area shape="rect" coords="100,0,200,100" href="bedroom.html">
</map>
```

#### DO

```html
<map name="floorplan">
  <area shape="rect" coords="0,0,100,100" href="kitchen.html" alt="Kitchen">
  <area shape="rect" coords="100,0,200,100" href="bedroom.html" alt="Bedroom">
</map>
```

---

### F30 - Don't use placeholder or filename text as an alt attribute

Text alternatives such as "image", "spacer", "picture1.jpg", or "DSC_0042" carry no meaning and fail as substitutes for the image content. (SC 1.1.1, 1.2.1)

#### DON'T

```html
<img src="quarterly-sales-chart.jpg" alt="chart.jpg">
```

#### DO

```html
<img src="quarterly-sales-chart.jpg" alt="Quarterly sales rose 12% from Q1 to Q2 2026.">
```

---

### F65 - Never omit the alt attribute entirely

Leaving off `alt` on `img`, `area`, or `input type="image"` elements causes assistive technology to fall back to announcing the filename or URL. Always include the attribute, even if empty. (SC 1.1.1)

#### DON'T

```html
<img src="team-photo.jpg">
```

#### DO

```html
<img src="team-photo.jpg" alt="The engineering team at the 2026 offsite">
```

---

## Forms & Labels

### H44 - Associate a label element with every form control

Every text input, textarea, select, checkbox, and radio button needs a programmatically associated `label` so assistive technology can announce its purpose. (SC 1.3.1, 4.1.2)

#### DON'T

```html
<p>Email address</p>
<input type="email" name="email">
```

#### DO

```html
<label for="email">Email address</label>
<input type="email" id="email" name="email">
```

---

### H65 - Use the title attribute only when a visible label element isn't possible

When a form control genuinely cannot have a visible `label` (e.g. a single search box with no room for text), fall back to a `title` attribute to give it an accessible name. (SC 1.3.1, 4.1.2)

#### DON'T

```html
<input type="search" name="q" placeholder="Search">
```

#### DO

```html
<input type="search" name="q" title="Search the site" placeholder="Search">
```

> Note: `placeholder` text disappears on input and is not a substitute for a label or title.

---

### H71 - Group related radio buttons/checkboxes with fieldset and legend

When several controls form one logical question (e.g. a set of radio buttons), wrap them in `fieldset` with a `legend` describing the group, so the relationship is announced. (SC 1.3.1, 4.1.2)

#### DON'T

```html
<p>Preferred contact method</p>
<input type="radio" name="contact" id="c1" value="email"><label for="c1">Email</label>
<input type="radio" name="contact" id="c2" value="phone"><label for="c2">Phone</label>
```

#### DO

```html
<fieldset>
  <legend>Preferred contact method</legend>
  <input type="radio" name="contact" id="c1" value="email"><label for="c1">Email</label>
  <input type="radio" name="contact" id="c2" value="phone"><label for="c2">Phone</label>
</fieldset>
```

---

### H90 - Indicate required form fields in the label or legend

Required fields must be identified in a way that is exposed to assistive technology, not by color or an asterisk in the visual layout alone. (SC 1.3.1, 3.3.2)

#### DON'T

```html
<label for="lastname">Last name</label> <span style="color:red">*</span>
<input id="lastname" name="lastname">
```

#### DO

```html
<label for="lastname">Last name (required)</label>
<input id="lastname" name="lastname" required aria-required="true">
```

---

### F68 - Never leave an interactive control without a determinable accessible name

Custom widgets (buttons, toggles, comboboxes built from `div`/`span`) must expose a name via visible text, `aria-label`, or `aria-labelledby` — an icon or empty element alone is not enough. (SC 4.1.2)

#### DON'T

```html
<div class="btn" onclick="closeDialog()"><i class="icon-x"></i></div>
```

#### DO

```html
<button type="button" class="btn" onclick="closeDialog()" aria-label="Close dialog">
  <i class="icon-x" aria-hidden="true"></i>
</button>
```

---

### H98 - Use HTML autocomplete attributes to identify input purpose

Common input fields (name, email, address, etc.) should declare their purpose via the `autocomplete` attribute so users can rely on browser/AT autofill. (SC 1.3.5)

#### DON'T

```html
<label for="email">Email</label>
<input type="email" id="email" name="email">
```

#### DO

```html
<label for="email">Email</label>
<input type="email" id="email" name="email" autocomplete="email">
```

---

### G83 - Identify errors and describe how to fix them in text

When a submitted value is missing, invalid, or outside the allowed range, provide a text description of the error and what is expected — not just a red border or icon. (SC 3.3.1, 3.3.3)

#### DON'T

```html
<label for="dob">Date of birth</label>
<input id="dob" name="dob" style="border: 2px solid red">
```

#### DO

```html
<label for="dob">Date of birth (MM/DD/YYYY)</label>
<input id="dob" name="dob" aria-invalid="true" aria-describedby="dob-error">
<p id="dob-error" role="alert">Please enter your date of birth in the format MM/DD/YYYY.</p>
```

---

## Headings & Document Structure

### H42 - Mark up headings using h1-h6 elements

Section headings must use real heading elements, not styled paragraphs or bold text, so assistive technology can build a navigable outline of the page. (SC 1.3.1)

#### DON'T

```html
<p><strong style="font-size:1.5em">Shipping information</strong></p>
```

#### DO

```html
<h2>Shipping information</h2>
```

---

### F43 - Don't fake headings with visual styling alone

Applying large/bold font styling to a line of text does not make it a heading to assistive technology if no heading element is used. (SC 1.3.1)

#### DON'T

```html
<div class="fake-heading">Order Summary</div>
```

#### DO

```html
<h2 class="fake-heading">Order Summary</h2>
```

---

### G141 - Keep heading levels in a logical, non-skipping order

Headings should nest sequentially (h1 → h2 → h3) to reflect document structure; don't skip levels or use heading rank for visual size alone. (SC 1.3.1, 2.4.6, 2.4.10)

#### DON'T

```html
<h1>Product page</h1>
<h4>Specifications</h4>
<h2>Reviews</h2>
```

#### DO

```html
<h1>Product page</h1>
<h2>Specifications</h2>
<h2>Reviews</h2>
```

---

## Language

### H57 - Declare the page language with the lang attribute

The `html` element must carry a valid `lang` attribute so screen readers select the correct pronunciation and voice. (SC 3.1.1)

#### DON'T

```html
<html>
<head><title>Bienvenue</title></head>
```

#### DO

```html
<html lang="fr">
<head><title>Bienvenue</title></head>
```

---

## Links

### H30 - Give links text that describes their destination or purpose

Link text (including alt text of a linked image) must make sense out of context, describing where the link goes or what it does. (SC 2.4.4, 2.4.9)

#### DON'T

```html
<a href="/annual-report-2026.pdf">Click here</a>
```

#### DO

```html
<a href="/annual-report-2026.pdf">Download the 2026 Annual Report (PDF)</a>
```

---

### LNK001 - Don't reuse identical link text for links with different destinations

The same visible text ("Read more", "Learn more") used for links that go to different places is ambiguous for users navigating by a list of links. The Understanding document for SC 2.4.9 states this as best practice: links with different purposes and destinations should have different descriptions. No technique or failure covers it (F84 is about non-specific text such as "click here" itself, which H30 covers). (SC 2.4.9)

#### DON'T

```html
<h3>Wireless Headphones</h3>
<a href="/products/1">Read more</a>
<h3>Bluetooth Speaker</h3>
<a href="/products/2">Read more</a>
```

#### DO

```html
<h3>Wireless Headphones</h3>
<a href="/products/1">Read more about Wireless Headphones</a>
<h3>Bluetooth Speaker</h3>
<a href="/products/2">Read more about Bluetooth Speaker</a>
```

---

## Color & Contrast

### G18 - Ensure body text has a contrast ratio of at least 4.5:1

Normal-size text and its background must meet a 4.5:1 contrast ratio so low-vision users can read it. (SC 1.4.3)

#### DON'T

```css
.body-text { color: #999999; background-color: #ffffff; } /* ratio ~2.85:1 */
```

#### DO

```css
.body-text { color: #595959; background-color: #ffffff; } /* ratio ~7:1 */
```

---

### G145 - Ensure large-scale text has a contrast ratio of at least 3:1

Text at 18pt (24px) or 14pt bold (~19px bold) and larger only needs a 3:1 contrast ratio against its background. (SC 1.4.3)

#### DON'T

```css
h1 { font-size: 28px; color: #aaaaaa; background-color: #ffffff; } /* ratio ~2:1 */
```

#### DO

```css
h1 { font-size: 28px; color: #767676; background-color: #ffffff; } /* ratio ~4.5:1 */
```

---

### G14 / F24 - Never use color as the only means of conveying information

If color is used to indicate meaning (required fields, form errors, status), pair it with text, an icon, or another visual cue so colorblind users don't miss it. (SC 1.4.1)

#### DON'T

```html
<p style="color: red">Please review the highlighted fields.</p>
<input style="border-color: red">
```

#### DO

```html
<p style="color: red">⚠ Please review the fields marked "Required" below.</p>
<label for="phone">Phone number (required)</label>
<input id="phone" style="border-color: red" aria-invalid="true">
```

---

## Tables

### H51 - Use table markup only for tabular data, with proper header cells

Data tables must use `table`, `tr`, `td`, and `th` elements (with `scope` or `headers`/`id` for complex tables); never use tables purely for visual layout with `th`-less rows. (SC 1.3.1)

#### DON'T

```html
<table>
  <tr><td><b>Month</b></td><td><b>Revenue</b></td></tr>
  <tr><td>January</td><td>$12,000</td></tr>
</table>
```

#### DO

```html
<table>
  <tr><th scope="col">Month</th><th scope="col">Revenue</th></tr>
  <tr><td>January</td><td>$12,000</td></tr>
</table>
```

---

### H63 - Use the scope attribute to bind header cells to data cells

For simple data tables, `scope="col"` or `scope="row"` on `th` elements tells assistive technology which data cells each header describes. (SC 1.3.1)

#### DON'T

```html
<table>
  <tr><th>Name</th><th>Score</th></tr>
  <tr><td>Alex</td><td>92</td></tr>
</table>
```

#### DO

```html
<table>
  <tr><th scope="col">Name</th><th scope="col">Score</th></tr>
  <tr><td>Alex</td><td>92</td></tr>
</table>
```

---

### F91 - Never leave table header cells unmarked in a data table

Using `td` for cells that function as row/column headers means screen reader users lose the header-data relationship entirely when navigating cell by cell. (SC 1.3.1)

#### DON'T

```html
<table>
  <tr><td>Product</td><td>Price</td></tr>
  <tr><td>Widget</td><td>$9.99</td></tr>
</table>
```

#### DO

```html
<table>
  <tr><th scope="col">Product</th><th scope="col">Price</th></tr>
  <tr><td>Widget</td><td>$9.99</td></tr>
</table>
```

---

## ARIA Usage

### ARIA16 - Use aria-labelledby to name controls from visible text elsewhere on the page

When a control's accessible name should come from text that isn't a `label`-compatible element, reference it with `aria-labelledby`. (SC 1.3.1, 4.1.2)

#### DON'T

```html
<span id="billing-heading">Billing address</span>
<input type="text" name="billing-address">
```

#### DO

```html
<span id="billing-heading">Billing address</span>
<input type="text" name="billing-address" aria-labelledby="billing-heading">
```

---

### ARIA14 - Use aria-label to name icon-only controls

When a control has no visible text label (icon-only buttons), provide an accessible name with `aria-label`. (SC 4.1.2)

#### DON'T

```html
<button><svg><!-- trash icon --></svg></button>
```

#### DO

```html
<button aria-label="Delete item"><svg aria-hidden="true"><!-- trash icon --></svg></button>
```

---

### ARIA1 - Use aria-describedby to attach descriptive text to controls

When a control needs more information than its label (instructions, format hints), reference the element holding that text with `aria-describedby`. The referenced id must exist in the same document. (SC 1.3.1, 3.3.2)

#### DON'T

```html
<label for="fname">First name</label>
<input id="fname" type="text" aria-describedby="fname-hint">
<!-- no element has id="fname-hint" -->
```

#### DO

```html
<label for="fname">First name</label>
<input id="fname" type="text" aria-describedby="fname-hint">
<p id="fname-hint">Your first name is sometimes called your "given name".</p>
```

---

### IDS001 - Never duplicate id attribute values on a page

Duplicate `id`s break `aria-labelledby`/`aria-describedby`/`for` references and `id`-based fragment navigation, since only the first match is used reliably. This was failure F77 of SC 4.1.1 Parsing, which WCAG 2.2 removed; its Understanding document notes that such issues now fail SC 1.3.1 or 4.1.2 instead. (SC 1.3.1, 4.1.2)

#### DON'T

```html
<label for="search">Search</label>
<input id="search" name="q">
...
<label for="search">Site search</label>
<input id="search" name="q2">
```

#### DO

```html
<label for="search-header">Search</label>
<input id="search-header" name="q">
...
<label for="search-footer">Site search</label>
<input id="search-footer" name="q2">
```

---

### ARIA5 - Keep ARIA state/property attributes in sync with the actual UI state

Widgets built with ARIA roles (`aria-expanded`, `aria-checked`, `aria-selected`, etc.) must have those attributes updated by script whenever the visual state changes. (SC 4.1.2)

#### DON'T

```html
<button aria-expanded="false" onclick="togglePanel()">More options</button>
<!-- togglePanel() shows/hides #panel but never updates aria-expanded -->
```

#### DO

```html
<button aria-expanded="false" onclick="togglePanel(this)">More options</button>
<script>
function togglePanel(btn) {
  const expanded = btn.getAttribute('aria-expanded') === 'true';
  btn.setAttribute('aria-expanded', String(!expanded));
  document.getElementById('panel').hidden = expanded;
}
</script>
```

---

## Multimedia

### G87 - Provide closed captions for prerecorded video with audio

Synchronized captions must be available for any prerecorded video that has a soundtrack, so deaf and hard-of-hearing users get the audio content. (SC 1.2.2)

#### DON'T

```html
<video src="training.mp4" controls></video>
```

#### DO

```html
<video src="training.mp4" controls>
  <track kind="captions" src="training-en.vtt" srclang="en" label="English">
</video>
```

---

### G93 - Provide audio description for prerecorded video

When on-screen visual information (actions, scene changes) isn't conveyed by the existing audio, provide an audio description track or a described version of the video. (SC 1.2.5)

#### DON'T

```html
<video src="product-demo.mp4" controls></video>
```

#### DO

```html
<video src="product-demo.mp4" controls>
  <track kind="descriptions" src="product-demo-desc.vtt" srclang="en" label="Audio description">
</video>
```

---

### G186 - Let users pause, stop, or hide moving, blinking, or scrolling content

Any content that moves, blinks, scrolls, or auto-updates for more than 5 seconds needs a visible control to pause, stop, or hide it. (SC 2.2.2)

#### DON'T

```html
<marquee>Breaking news: sale ends today! Breaking news: sale ends today!</marquee>
```

#### DO

```html
<div class="ticker" id="ticker" aria-live="off">Breaking news: sale ends today!</div>
<button type="button" onclick="document.getElementById('ticker').classList.toggle('paused')">
  Pause ticker
</button>
```

---

## Keyboard & Focus

### G202 - Ensure all functionality is operable from the keyboard alone

Interactive behavior implemented on `div`/`span` elements must respond to keyboard events (`Enter`/`Space`) and be focusable, not rely on `onclick`/`onmouseover` only. (SC 2.1.1)

#### DON'T

```html
<div onclick="submitForm()">Submit</div>
```

#### DO

```html
<button type="button" onclick="submitForm()">Submit</button>
```

Or, if a native element truly can't be used:

```html
<div role="button" tabindex="0" onclick="submitForm()"
     onkeydown="if(event.key==='Enter'||event.key===' '){submitForm();event.preventDefault();}">
  Submit
</div>
```

---

### G21 - Never trap keyboard focus in a component

A user tabbing into a widget (modal, menu, embedded player) must always be able to tab back out using only the keyboard, with no dead end. (SC 2.1.2)

#### DON'T

```html
<div id="modal" onkeydown="event.preventDefault()">
  <input type="text">
  <button>Close</button>
</div>
```

#### DO

```html
<div id="modal" role="dialog" aria-modal="true">
  <input type="text">
  <button onclick="closeModal()">Close</button>
</div>
<!-- Escape key and Tab/Shift+Tab cycle normally within the dialog and can close it -->
```

---

### G195 - Provide a visible focus indicator on all interactive elements

Every focusable element needs a visible outline or highlight when it receives keyboard focus; never remove the default focus ring without a visible replacement. (SC 2.4.7)

#### DON'T

```css
a:focus, button:focus, input:focus { outline: none; }
```

#### DO

```css
a:focus-visible, button:focus-visible, input:focus-visible {
  outline: 3px solid #1a73e8;
  outline-offset: 2px;
}
```

---

### FOC001 - Don't let sticky headers/footers fully hide the focused element

When a `user interface component` receives keyboard focus, no author-created content (e.g. a sticky header, cookie banner, or footer) may completely obscure it. (SC 2.4.11 — Focus Not Obscured (Minimum), new in WCAG 2.2)

#### DON'T

```css
.sticky-header { position: fixed; top: 0; height: 80px; z-index: 10; }
/* focused links scrolled to the top of the page end up entirely underneath the header */
```

#### DO

```css
.sticky-header { position: fixed; top: 0; height: 80px; z-index: 10; }
main { scroll-margin-top: 90px; } /* keeps focused elements clear of the fixed header */
```

---

## Navigation

### G1 - Provide a skip link to bypass repeated content

A link at the very start of the page should let keyboard users jump directly to the main content, skipping repeated navigation/header blocks. (SC 2.4.1)

#### DON'T

```html
<body>
  <nav><!-- 40 navigation links --></nav>
  <main>...</main>
</body>
```

#### DO

```html
<body>
  <a class="skip-link" href="#main-content">Skip to main content</a>
  <nav><!-- 40 navigation links --></nav>
  <main id="main-content">...</main>
</body>
```

---

### G61 - Keep repeated navigation in the same relative order on every page

Navigation bars, search boxes, and other components that repeat across pages in a site must appear in the same order each time, unless the user changes it. (SC 3.2.3)

#### DON'T

```html
<!-- page-a.html -->
<nav><a href="/">Home</a><a href="/shop">Shop</a><a href="/about">About</a></nav>
<!-- page-b.html -->
<nav><a href="/about">About</a><a href="/">Home</a><a href="/shop">Shop</a></nav>
```

#### DO

```html
<!-- Both pages use the identical order -->
<nav><a href="/">Home</a><a href="/shop">Shop</a><a href="/about">About</a></nav>
```

---

### HLP001 - Keep help mechanisms in the same relative location across pages

If a set of pages offers a help mechanism (contact link, live chat, help page link, self-help option), it must appear in the same relative order/place on every page it's provided, unless the user moves it. (SC 3.2.6 — Consistent Help, new in WCAG 2.2)

#### DON'T

```html
<!-- checkout.html -->
<footer><a href="/help">Help</a></footer>
<!-- account.html -->
<header><a href="/help">Help</a></header>
```

#### DO

```html
<!-- Both pages place Help in the same relative position, e.g. always in the header -->
<header><nav>...<a href="/help">Help</a></nav></header>
```

---

## Timing & Motion

### G133 - Warn users before a session times out and let them extend it

If a time limit is enforced (session expiry), users must be warned before it expires and given a simple way to extend it, unless the time limit is essential. (SC 2.2.1)

#### DON'T

```js
setTimeout(() => { window.location.href = '/login?expired=true'; }, 10 * 60 * 1000);
```

#### DO

```js
setTimeout(() => {
  showDialog('Your session will expire in 60 seconds.', {
    onExtend: () => resetSessionTimer(),
  });
}, 9 * 60 * 1000);
```

---

### ANI001 - Provide a way to disable motion triggered by interaction

Animations triggered by user interaction (parallax, sliding panels, zoom transitions) must be disable-able, and should respect the OS-level "reduce motion" preference. (SC 2.3.3)

#### DON'T

```css
.card { transition: transform 0.6s; }
.card:hover { transform: scale(1.3) rotate(5deg); }
```

#### DO

```css
.card { transition: transform 0.6s; }
.card:hover { transform: scale(1.3) rotate(5deg); }

@media (prefers-reduced-motion: reduce) {
  .card { transition: none; }
  .card:hover { transform: none; }
}
```

---

## Pointer & Target Size (WCAG 2.2)

### DRG001 - Provide a non-dragging alternative for drag-operated functionality

Any function operated by a dragging gesture (reordering a list, a slider, a carousel swipe) must also be achievable with a single pointer action that doesn't require dragging (e.g. tap-to-select buttons, up/down controls). (SC 2.5.7 — Dragging Movements, new in WCAG 2.2)

#### DON'T

```html
<div class="slider-track" id="volume" draggable="true" ondragend="setVolume(event)"></div>
<!-- volume can only be changed by dragging the handle -->
```

#### DO

```html
<input type="range" id="volume" min="0" max="100" value="50" aria-label="Volume">
<!-- native range input supports drag, arrow keys, and click-to-set -->
```

---

### TGT001 - Make pointer targets at least 24x24 CSS pixels

Interactive elements operated by pointer must have a target size of at least 24x24 CSS pixels, or sufficient spacing from neighboring targets, unless an exception (inline text, essential, or equivalent control available) applies. (SC 2.5.8 — Target Size (Minimum), new in WCAG 2.2)

#### DON'T

```css
.icon-button { width: 16px; height: 16px; padding: 0; margin: 2px; }
```

#### DO

```css
.icon-button { width: 24px; height: 24px; padding: 8px; margin: 4px; }
/* effective clickable area is at least 24x24px with spacing from adjacent targets */
```

---

## Cognitive & Authentication (WCAG 2.2)

### RED001 - Don't make users re-enter information they already provided

Information a user already entered earlier in the same process (e.g. shipping address reused for billing) must be auto-populated or selectable, not required to be typed again from scratch. (SC 3.3.7 — Redundant Entry, new in WCAG 2.2)

#### DON'T

```html
<h2>Billing address</h2>
<label for="b-street">Street</label><input id="b-street" name="b-street">
<label for="b-city">City</label><input id="b-city" name="b-city">
```

#### DO

```html
<h2>Billing address</h2>
<label><input type="checkbox" id="same-as-shipping" onchange="copyShippingToBilling(this)">
  Same as shipping address
</label>
<label for="b-street">Street</label><input id="b-street" name="b-street" autocomplete="billing street-address">
<label for="b-city">City</label><input id="b-city" name="b-city" autocomplete="billing address-level2">
```

---

### AUT001 - Don't require a cognitive function test for authentication without an alternative

Login flows must not rely solely on remembering a password, solving a puzzle, or transcribing a CAPTCHA unless an alternative is offered (e.g. password managers/paste allowed, email magic link, biometric option, or object-recognition-free CAPTCHA). (SC 3.3.8 — Accessible Authentication (Minimum), new in WCAG 2.2)

#### DON'T

```html
<label for="pwd">Password</label>
<input id="pwd" type="password" onpaste="return false">
```

#### DO

```html
<label for="pwd">Password</label>
<input id="pwd" type="password" autocomplete="current-password">
<!-- pasting is allowed so a password manager can fill it in -->
<p><a href="/login/magic-link">Or email me a sign-in link instead</a></p>
```

---

## Resize & Reflow

### C32 - Support reflow at 400% zoom without two-dimensional scrolling

Layouts must reflow to a single column at high zoom levels / narrow viewports so users don't need to scroll horizontally to read content. (SC 1.4.10)

#### DON'T

```css
.layout { display: flex; width: 1280px; } /* fixed width forces horizontal scrolling when zoomed */
```

#### DO

```css
.layout { display: flex; flex-wrap: wrap; max-width: 100%; }
@media (max-width: 400px) {
  .layout { flex-direction: column; }
}
```

---

### C35 - Don't block user-applied text spacing overrides

Content must remain readable (no clipped or overlapping text) when a user overrides line-height, paragraph spacing, letter spacing, and word spacing via a stylesheet override. (SC 1.4.12)

#### DON'T

```css
.card { height: 60px; overflow: hidden; line-height: 1; } /* text gets clipped when spacing is increased */
```

#### DO

```css
.card { min-height: 60px; overflow: visible; line-height: 1.5; }
```

---

### G142 - Support browser zoom by using relative units, not fixed pixel text

Text sized in `px` can fail to scale in some browser zoom/text-only-zoom modes; use `rem`/`em`/`%` so text resizes properly up to 200% without loss of content or function. (SC 1.4.4)

#### DON'T

```css
body { font-size: 14px; }
h1 { font-size: 24px; }
```

#### DO

```css
html { font-size: 100%; } /* 1rem = 16px by default, respects user's browser font-size setting */
body { font-size: 0.875rem; }
h1 { font-size: 1.5rem; }
```

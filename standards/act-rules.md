# ACT Rules Accessibility Checker Rules

This document lists the W3C-approved [ACT Rules](https://www.w3.org/WAI/standards-guidelines/act/rules/) (Accessibility Conformance Testing), written in the [ACT Rules Format 1.1](https://www.w3.org/TR/act-rules-format/) as checked in [w3c/wcag-act](https://github.com/w3c/wcag-act) at commit `db08178` (February 2026). Rule text, requirements and examples come from `_rules/` in [act-rules/act-rules.github.io](https://github.com/act-rules/act-rules.github.io) at commit `7e46818` (September 2026).

Only the 37 approved rules are listed: those marked `"proposed": false` and `"deprecated": false` in [w3c/wcag-act-rules](https://github.com/w3c/wcag-act-rules) at commit `415d353` (September 2026). Rules are grouped by category and use the ACT rule's own 6-character ID. Each entry cites the WCAG Success Criteria (SC) it supports.

An `Automation:` line says what a static render can't check, an `Overlaps:` line names the entry in [wcag2.2-rules.md](./wcag2.2-rules.md) covering the same check, and a `Prevailing rule:` line names the rule that wins a conflict in [overlap.md](./overlap.md). A `Source:` line links the rule's page.

## Table of Contents

1. [Page Title & Language](#page-title--language)
2. [Text Alternatives (Images & Objects)](#text-alternatives-images--objects)
3. [Controls, Links & Form Fields](#controls-links--form-fields)
4. [Tables](#tables)
5. [ARIA Usage](#aria-usage)
6. [Keyboard & Focus](#keyboard--focus)
7. [Color & Contrast](#color--contrast)
8. [Timing](#timing)
9. [Zoom, Text Spacing & Orientation](#zoom-text-spacing--orientation)

---

## Page Title & Language

### 2779a5 - Give every page a non-empty title element

The page must have a `title` element whose text is not empty or only whitespace. Only the first `title` counts. (SC 2.4.2)

Overlaps: H25 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/2779a5/>

#### DON'T

```html
<html>
  <h1>this page has no title</h1>
</html>

<html>
  <title></title>
</html>
```

#### DO

```html
<html>
  <title>This page has a title</title>
</html>
```

---

### c4a8a4 - Give every page a title that describes its topic or purpose

The page's first `title` must describe the page's topic or purpose; a generic title, such as the site name alone, fails. (SC 2.4.2)

Automation: whether the title describes the page needs manual / AI review.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/c4a8a4/>

#### DON'T

```html
<html lang="en">
  <head>
    <title>Apple harvesting season</title>
  </head>
  <body>
    <p>
      Clementines will be ready to harvest from late October through February.
    </p>
  </body>
</html>
```

#### DO

```html
<html lang="en">
  <head>
    <title>Clementine harvesting season</title>
  </head>
  <body>
    <p>
      Clementines will be ready to harvest from late October through February.
    </p>
  </body>
</html>
```

---

### b5c3f8 - Give the html element a non-empty lang attribute

The `html` element of a top-level HTML page must have a `lang` attribute that is not empty or only whitespace. (SC 3.1.1)

Overlaps: H57 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/b5c3f8/>

#### DON'T

```html
<html>
  <body>
    The quick brown fox jumps over the lazy dog.
  </body>
</html>

<html lang=" ">
  <body>
    The quick brown fox jumps over the lazy dog.
  </body>
</html>
```

#### DO

```html
<html lang="en">
  <body>
    The quick brown fox jumps over the lazy dog.
  </body>
</html>
```

---

### bf051a - Use a known primary language subtag in the html element's lang attribute

The `html` element's `lang` must start with a primary language subtag listed in the IANA Language Subtag Registry; other subtags are ignored (`en-US-GB` passes), and three-letter codes such as `eng` fail. (SC 3.1.1)

Automation: needs a copy of the registry's primary language subtags.

Overlaps: H57 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/bf051a/>

#### DON'T

```html
<html lang="em-US"></html>
<html lang="#1"></html>
```

#### DO

```html
<html lang="FR"></html>
<html lang="en-US-GB"></html>
```

---

### de46e4 - Use a known primary language subtag in every lang attribute in the body

Every `lang` attribute on an element in the `body` that contains text must have a known primary language subtag, as for `bf051a`; a value of only whitespace fails. (SC 3.1.2)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/de46e4/>

#### DON'T

```html
<html lang="es">
  <body>
    <article lang="dutch">
      Zij liepen een vreemde Tiki bar binnen, aan de rand van een dorpje aan het strand.
    </article>
  </body>
</html>
```

#### DO

```html
<html lang="en">
  <body>
    <blockquote lang="fr-CH">
      Ils ont trouvé un étrange bar Tiki aux abords de la petite ville balnéaire.
    </blockquote>
  </body>
</html>
```

---

## Text Alternatives (Images & Objects)

### 23a2a8 - Give every image a non-empty accessible name or mark it as decorative

Every `img` and `role="img"` element that isn't hidden must have a non-empty accessible name, or be marked decorative with `alt=""` or `role="none"`/`"presentation"`. (SC 1.1.1)

Overlaps: H37, F65 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/23a2a8/>

#### DON'T

```html
<img src="/test-assets/shared/w3c-logo.png" />
<div role="img" style="width:72px; height:48px; background-image: url(/test-assets/shared/w3c-logo.png)"></div>
```

#### DO

```html
<img alt="W3C logo" src="/test-assets/shared/w3c-logo.png" />
<img alt="" src="/test-assets/shared/background.png" />
```

---

### qt1vmo - Give images an accessible name that serves the same purpose as the image

The accessible name of a visible `img`, `canvas` or `svg` must serve the same purpose as the image, unless the image is inside a named link or control, or didn't load. (SC 1.1.1)

Automation: judging whether the name describes the image needs manual / AI review, and litehtml never loads images.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/qt1vmo/>

#### DON'T

```html
<img src="/test-assets/shared/w3c-logo.png" alt="ERCIM logo" />
```

#### DO

```html
<img src="/test-assets/shared/w3c-logo.png" alt="W3C logo" />
```

---

### 0va7u6 - Don't use images of text

Don't use images (`img`, `input type="image"`, images in `svg`, CSS background images) that contain visible text, unless the image is decorative, the text is incidental or essential, or the same text is also available as real text. (SC 1.4.5; SC 1.4.9 secondary)

Automation: finding text in an image and judging the exceptions needs manual / AI review; litehtml never loads images.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/0va7u6/>

#### DON'T

```html
<img
  src="/test-assets/0va7u6/textimage.jpg"
  alt="The Accessibility Conformance Testing (ACT) Rules Format 1.0 defines a format for writing accessibility test rules."
/>

<input type="image" src="/test-assets/0va7u6/button.jpg" alt="Press me" />
```

#### DO

```html
<img src="/test-assets/shared/fireworks.jpg" alt="fireworks going off behind the Eiffel tower at night" />
<input type="image" src="/test-assets/shared/file.svg" alt="New file" />
```

---

### 46ca7f - Don't expose elements marked as decorative

Don't give an element marked decorative (`role="none"`/`"presentation"`, or an `img` with `alt=""`) a global ARIA attribute such as a non-empty `aria-label` or `aria-labelledby`, or make it focusable: either one exposes it again with its implicit role. (SC 1.1.1 secondary)

Overlaps: H67 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/46ca7f/>

#### DON'T

```html
<img src="/test-assets/shared/w3c-logo.png" alt="" aria-labelledby="label" /> <span hidden id="label">W3C logo</span>

<svg role="none" aria-label="Yellow circle">
  <circle cx="50" cy="50" r="40" fill="yellow"></circle>
</svg>
```

#### DO

```html
<img src="/test-assets/shared/w3c-logo.png" alt="" />
<img src="/test-assets/shared/w3c-logo.png" alt="" aria-hidden="true" />
```

---

### 59796f - Give image buttons an accessible name other than the default

Every `input type="image"` must have an accessible name (from `alt`, `aria-label`, `aria-labelledby` or `title`) that is neither empty nor the browser's default "Submit Query"; the `name` attribute doesn't count. (SC 1.1.1, 4.1.2)

Overlaps: H36 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/59796f/>

#### DON'T

```html
<input type="image" name="search" src="/test-assets/shared/search-icon.svg" />
<input type="image" src="/test-assets/shared/search-icon.svg" alt="" />
```

#### DO

```html
<input type="image" src="/test-assets/shared/search-icon.svg" alt="Search" />
```

---

### 7d6734 - Give SVG elements with an image role a non-empty accessible name

Every `svg` element with a `role` of `img`, `graphics-document` or `graphics-symbol` must have a non-empty accessible name, from a `title` child, `aria-label` or `aria-labelledby`. (SC 1.1.1)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/7d6734/>

#### DON'T

```html
<svg xmlns="http://www.w3.org/2000/svg" role="img">
  <circle cx="50" cy="50" r="40" stroke="green" stroke-width="4" fill="yellow"></circle>
</svg>

<svg xmlns="http://www.w3.org/2000/svg" role="img">
  <title></title>
  <circle cx="50" cy="50" r="40" fill="yellow"></circle>
</svg>
```

#### DO

```html
<svg xmlns="http://www.w3.org/2000/svg" role="img" width="100" height="100">
  <title>1 circle</title>
  <circle cx="50" cy="50" r="40" fill="yellow"></circle>
</svg>
```

---

### 8fc3b6 - Give object elements that embed images, audio or video a non-empty accessible name

Every `object` element with no explicit role that embeds an image, audio or video must have a non-empty accessible name, from `title`, `aria-label` or `aria-labelledby`. (SC 1.1.1)

Automation: the embedded resource's MIME type isn't in the render, so it has to come from the `type` attribute, the `data` URL's extension, or a separate request.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/8fc3b6/>

#### DON'T

```html
<object data="/test-assets/moon-audio/moon-speech.mp3"></object>
<object title="" data="/test-assets/rabbit-video/video.mp4"></object>
```

#### DO

```html
<object aria-label="Moon speech" data="/test-assets/moon-audio/moon-speech.mp3"></object>
<object title="Rabbit animated short" data="/test-assets/rabbit-video/video.mp4"></object>
```

---

## Controls, Links & Form Fields

### 97a4e1 - Give every button a non-empty accessible name

Every element with a `button` role, except `input type="image"` (see `59796f`), must have a non-empty accessible name. `value` names an `input` button, but not a `button` element. (SC 4.1.2)

Overlaps: F68 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/97a4e1/>

#### DON'T

```html
<button></button>
<button type="button" value="read more"></button>
<span role="button"></span>
```

#### DO

```html
<button>My button</button>
<input type="submit" value="Submit" />
<button aria-label="My button"></button>
```

---

### c487ae - Give every link a non-empty accessible name

Every link (`a` or `area` with an `href`, or `role="link"`) must have a non-empty accessible name; a link whose only content is a decorative image has none. (SC 2.4.4, 2.4.9, 4.1.2; SC 1.1.1 secondary)

Overlaps: H30, F89 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/c487ae/>

#### DON'T

```html
<a href="http://www.w3.org/WAI"></a>
<a href="https://www.w3.org/WAI"><img src="/test-assets/shared/w3c-logo.png" alt=""/></a>
```

#### DO

```html
<a href="https://www.w3.org/WAI"> Web Accessibility Initiative (WAI) </a>
```

---

### m6b1q3 - Give every menuitem a non-empty accessible name

Every element with a `menuitem` role must have a non-empty accessible name; a decorative icon alone gives none. (SC 4.1.2)

Overlaps: F68 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/m6b1q3/>

#### DON'T

```html
<div role="menu">
  <button role="menuitem">
    <img src="/test-assets/shared/file.svg" alt="" />
  </button>
</div>
```

#### DO

```html
<div role="menu">
  <button role="menuitem" aria-label="New file">
    <img src="/test-assets/shared/file.svg" alt="" />
  </button>
</div>
```

---

### 2t702h - Give every summary element a non-empty accessible name

The first `summary` of every `details` element, including one with `role="none"`, must have a non-empty accessible name other than just its marker. (SC 4.1.2)

Overlaps: F68 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/2t702h/>

#### DON'T

```html
<details>
  <summary></summary>
  <p>This is a website. We are available 24/7.</p>
</details>

<details>
  <summary role="none"></summary>
  <p>This is a website. We are available 24/7.</p>
</details>
```

#### DO

```html
<details>
  <summary>Opening times</summary>
  <p>This is a website. We are available 24/7.</p>
</details>
```

---

### e086e5 - Give every form field a non-empty accessible name

Every form field must have a non-empty accessible name, disabled ones included: elements with a `checkbox`, `combobox`, `listbox`, `menuitemcheckbox`, `menuitemradio`, `radio`, `searchbox`, `slider`, `spinbutton`, `switch` or `textbox` role, and `input` elements of type `color`, `date`, `datetime-local`, `file`, `month`, `password`, `time` or `week`. An `aria-label` of only a space is empty. (SC 4.1.2; SC 1.3.1, 2.5.3 secondary)

Overlaps: H44, F68 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/e086e5/>

#### DON'T

```html
<div>last name</div>
<input />

<input aria-label=" " />
```

#### DO

```html
<label>
  first name
  <input />
</label>

<label for="country">Country</label>
<select id="country">
  <option>England</option>
  <option>Scotland</option>
</select>
```

---

### 73f2c2 - Use valid autofill tokens in autocomplete attributes

An `autocomplete` value other than `on` or `off` must be, case-insensitively and in this order: an optional `section-*`, an optional `shipping` or `billing`, an optional `home`, `work`, `mobile`, `fax` or `pager` (only before `email`, `impp`, `tel` or `tel-*`), one autofill field name, and an optional `webauthn`. Disabled or hidden fields, `input`s of type `button`, `checkbox`, `file`, `image`, `radio`, `reset` or `submit`, and non-focusable elements with a non-widget role are exempt. (SC 1.3.5)

Overlaps: F107, H98 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/73f2c2/>

#### DON'T

```html
<label>Username<input autocomplete="badname"/></label>
<label>Photo<input autocomplete="work photo"/></label>
<label>Email<input autocomplete="work shipping email"/></label>
```

#### DO

```html
<label>Username<input autocomplete="username"/></label>
<label> Street address<textarea autocomplete="Street-Address"></textarea></label>
```

---

## Tables

### a25f45 - Make headers attributes refer only to other cells in the same table

Every token in a table cell's `headers` attribute must be the `id` of another cell in the same table. (SC 1.3.1)

Overlaps: F90 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/a25f45/>

#### DON'T

```html
<table>
  <tr>
    <th id="headerOfColumn1">Projects</th>
    <th id="headerOfColumn2">Objective</th>
  </tr>
  <tr>
    <td headers="headOfColumn1">15%</td>
    <td headers="headOfColumn2">10%</td>
  </tr>
</table>
```

#### DO

```html
<table>
  <thead>
    <tr>
      <th id="header1">Projects</th>
      <th id="header2">Objective</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td headers="header1">15%</td>
      <td headers="header2">10%</td>
    </tr>
  </tbody>
</table>
```

---

## ARIA Usage

### 5f99a7 - Only use aria- attributes defined in WAI-ARIA

Every `aria-*` attribute must be defined in WAI-ARIA 1.2, the Graphics module or the DPub module; misspelled or invented names fail. (SC 1.3.1, 4.1.2 secondary)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/5f99a7/>

#### DON'T

```html
<div role="checkbox" aria-not-checked="true">All conditions are met</div>
```

#### DO

```html
<article aria-atomic="true">This is a description of something cool...</article>
```

---

### 6a7281 - Give ARIA states and properties a value that is valid for their value type

Every non-empty ARIA state or property must have a value that is valid for its type (`true/false`, `tristate`, token, number, ...); ID references don't have to resolve. (WAI-ARIA 1.2, 6.2.4 Value; SC 1.3.1, 4.1.2 secondary)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/6a7281/>

#### DON'T

```html
<div role="textbox" aria-required="undefined" aria-label="A required textbox"></div>
<div role="button" aria-expanded="collapsed">A button</div>
```

#### DO

```html
<div role="textbox" aria-required="true" aria-label="Family name"></div>
<div role="button" aria-expanded="undefined">A button</div>
```

---

### 674b10 - Use at least one valid, non-abstract WAI-ARIA role in every role attribute

Every non-empty `role` attribute on an element that isn't hidden must contain at least one valid, non-abstract WAI-ARIA role; the other tokens are fallbacks. (SC 1.3.1, 4.1.2 secondary)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/674b10/>

#### DON'T

```html
I love <span class="link" onclick="location.href='https://act-rules.github.io/'" role="lnik">ACT rules</span>.
```

#### DO

```html
<label>Search: <input type="text" role="searchbox" placeholder="Enter 3 or more characters"/></label>
<label>Search: <input type="text" role="searchfield searchbox" placeholder="Enter 3 or more characters"/></label>
```

---

### 4e8ab6 - Set every required state and property of an element's explicit role

An element with an explicit role that differs from its implicit role must set every state and property WAI-ARIA requires for that role, unless it has a default value: `heading` needs `aria-level`, `checkbox` and `switch` need `aria-checked`. (WAI-ARIA 1.2, 5.2.2 Required States and Properties; SC 1.3.1, 4.1.2 secondary)

Overlaps: ARIA5 in wcag2.2-rules.md (which covers keeping the state in sync by script; this rule only checks that it is set)

Prevailing rule: ARIA-USAGE001 in wai-aria-1.2-rules.md (see C6 in [overlap.md](./overlap.md#conflicts)). The custom `div role="checkbox"` DO example shows valid ARIA, not the recommended control; prefer a native `input type="checkbox"`.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/4e8ab6/>

#### DON'T

```html
<div role="heading">
  My First Heading
</div>

<div role="checkbox" aria-labelledby="label"></div>
<div id="label">Check me</div>
```

#### DO

```html
<div role="heading" aria-level="1">
  My First Heading
</div>

<div role="checkbox" aria-checked="false" aria-labelledby="label"></div>
<div id="label">Check me</div>
```

---

### 6cfa84 - Don't put focusable content inside aria-hidden="true"

Neither an element with `aria-hidden="true"` nor its descendants may be in the sequential focus order. `display:none` or `disabled` is fine; moving the content off screen, `aria-disabled`, or a nested `aria-hidden="false"` is not. (SC 4.1.2)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/6cfa84/>

#### DON'T

```html
<div aria-hidden="true">
  <a href="/" style="position:absolute; top:-999em">Link</a>
</div>

<div aria-hidden="true">
  <div aria-hidden="false">
    <button>Some button</button>
  </div>
</div>
```

#### DO

```html
<div aria-hidden="true">
  <a href="/" style="display:none">Link</a>
</div>

<input disabled aria-hidden="true" />
```

---

### 307n5z - Don't put focusable content inside elements whose children are presentational

Elements whose role makes their children presentational (`button`, `checkbox`, `img`, `meter`, `menuitemcheckbox`, `menuitemradio`, `option`, `progressbar`, `radio`, `scrollbar`, `separator`, `slider`, `switch`, `tab`) must not contain anything in the sequential focus order. (SC 4.1.2)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/307n5z/>

#### DON'T

```html
<p role="checkbox" aria-checked="false" tabindex="0">I agree to the <a href="/terms">terms of service</a></p>
```

#### DO

```html
<p id="terms">
  <span role="checkbox" aria-checked="false" tabindex="0" aria-labelledby="terms">
    I agree to the
  </span>
  <a href="/terms">terms of service</a>
</p>
```

---

## Keyboard & Focus

### oj04fd - Give every element in sequential focus order a visible focus indication

Every element in the sequential focus order must change at least one visible pixel when focused; removing the focus outline with CSS and adding nothing else fails. (SC 2.4.7)

Automation: needs each element rendered focused and unfocused, which litehtml can't do.

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/oj04fd/>

#### DON'T

```html
<link rel="stylesheet" href="/test-assets/focus-visible-oj04fd/styles.css" />
<a class="no-focus-default" href="https://act-rules.github.io/">ACT rules</a>
```

#### DO

```html
<a href="https://act-rules.github.io/">ACT rules</a>
<span tabindex="0">Act rules</span>
```

---

### akn7bn - Don't give an iframe with focusable content a negative tabindex

An `iframe` whose document contains visible focusable content must not have a negative `tabindex`. (SC 2.1.1, 2.1.3)

Automation: needs the nested document, which the renderer doesn't load (parse `srcdoc`, or fetch `src`).

Overlaps: G202 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/akn7bn/>

#### DON'T

```html
<iframe tabindex="-1" srcdoc="<a href='/'>Home</a>"></iframe>
```

#### DO

```html
<iframe srcdoc="<a href='/'>Home</a>"></iframe>
<iframe tabindex="0" srcdoc="<a href='/'>Home</a>"></iframe>
```

---

### 0ssw9k - Make scrollable regions reachable with sequential focus navigation

An element that scrolls (`overflow` of `auto` or `scroll`, with content larger than its box) must be focusable itself or contain a focusable element, unless it is inert. (SC 2.1.1, 2.1.3)

Automation: overflow caused by text wrapping is only an estimate, since text measurement is approximate.

Overlaps: G202 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/0ssw9k/>

#### DON'T

```html
<section style="height: 100px; width: 500px; overflow-y: scroll">
  <h1>WCAG 2.1 Abstract</h1>
  <p>
    Web Content Accessibility Guidelines (WCAG) 2.1 covers a wide range of recommendations for making Web content more
    accessible. ...
  </p>
</section>
```

#### DO

```html
<section style="height: 100px; width: 500px; overflow: scroll;" tabindex="0">
  <h1>WCAG 2.1 Abstract</h1>
  <p>
    Web Content Accessibility Guidelines (WCAG) 2.1 covers a wide range of recommendations for making Web content more
    accessible. ...
  </p>
</section>
```

---

## Color & Contrast

### afw4f7 - Give text a contrast ratio of at least 4.5:1, or 3:1 for large text

Visible text must have a contrast ratio of at least 4.5:1 with its background, or 3:1 for large text, unless it is decorative, not in a human language, or part of a disabled widget. (SC 1.4.3; SC 1.4.6 secondary)

Automation: the decorative / human-language exception needs human judgement, and backgrounds from images the renderer doesn't load can't be measured.

Overlaps: G18, G145 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/afw4f7/>

#### DON'T

```html
<p style="color: #AAA; background: white;">
  Some text in English
</p>
```

#### DO

```html
<p style="color: #333; background: #FFF;">
  Some text in a human language
</p>
```

---

### 09o5cg - Give text a contrast ratio of at least 7:1, or 4.5:1 for large text (enhanced)

The same as `afw4f7`, at Level AAA: at least 7:1, or 4.5:1 for large text. (SC 1.4.6; SC 1.4.3 secondary)

Automation: the same as for `afw4f7`.

Overlaps: G18, G145 in wcag2.2-rules.md (at the lower Level AA thresholds)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/09o5cg/>

#### DON'T

```html
<p style="color: #666; background: white;">
  Some text in English
</p>

<p style="color: #000; font-size:18pt; background: #666;">
  Some text in a human language
</p>
```

#### DO

```html
<p style="color: #333; background: #FFF;">
  Some text in a human language
</p>
```

---

## Timing

### bc659a - Only use a meta refresh that is instant or longer than 20 hours

The first valid `meta http-equiv="refresh"` must have a delay of 0 or more than 72000 seconds (20 hours), whether it reloads the page or redirects. (SC 2.2.1; SC 2.2.4, 3.2.5 secondary)

Overlaps: F40 / F41 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/bc659a/>

#### DON'T

```html
<head>
  <meta http-equiv="refresh" content="30" />
</head>

<head>
  <meta http-equiv="refresh" content="0: https://w3.org" />
  <meta http-equiv="refresh" content="5; https://w3.org" />
</head>
```

#### DO

```html
<head>
  <meta http-equiv="refresh" content="0; URL='https://github.com'" />
</head>

<head>
  <meta http-equiv="refresh" content="72001; https://w3.org" />
</head>
```

---

### bisz58 - Only use an instant meta refresh (no exception)

The first valid `meta http-equiv="refresh"` must have a delay of 0; unlike `bc659a`, a delay over 20 hours fails. (SC 2.2.4, 3.2.5; SC 2.2.1 secondary)

Overlaps: F40 / F41 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/bisz58/>

#### DON'T

```html
<head>
  <meta http-equiv="refresh" content="72001; URL='https://w3.org'" />
</head>
```

#### DO

```html
<head>
  <meta http-equiv="refresh" content="0; URL='https://w3.org'" />
</head>
```

---

## Zoom, Text Spacing & Orientation

### b4f0c3 - Don't disable zoom in the viewport meta element

In a viewport `meta` element, `user-scalable` must be absent, `yes`, `device-width`, `device-height` or a number not between -1 and 1, and `maximum-scale` must be absent, `device-width`, `device-height`, negative, or at least 2. (SC 1.4.4; SC 1.4.10 secondary)

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/b4f0c3/>

#### DON'T

```html
<meta name="viewport" content="user-scalable=no" />
<meta name="viewport" content="user-scalable=yes, initial-scale=0.8, maximum-scale=1.5" />
```

#### DO

```html
<meta name="viewport" content="user-scalable=yes" />
<meta name="viewport" content="maximum-scale=2.0" />
```

---

### 24afc2 - Don't lock letter spacing below 0.12 times the font size with !important in a style attribute

A `letter-spacing` set with `!important` in a `style` attribute, on an element with visible text, must be at least 0.12 times the font size (`normal` counts as 0). (SC 1.4.12)

Automation: parse the `style` attribute's declarations and compare them with the computed font size.

Overlaps: C35 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/24afc2/>

#### DON'T

```html
<p style="letter-spacing: 0.1em !important">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

#### DO

```html
<p style="letter-spacing: 0.15em !important">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

---

### 9e45ec - Don't lock word spacing below 0.16 times the font size with !important in a style attribute

The same as `24afc2` for `word-spacing`: at least 0.16 times the font size. (SC 1.4.12)

Overlaps: C35 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/9e45ec/>

#### DON'T

```html
<p style="word-spacing: normal !important">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

#### DO

```html
<p style="word-spacing: 0.2em !important">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

---

### 78fd32 - Don't lock line height below 1.5 times the font size with !important in a style attribute

The same as `24afc2` for `line-height`, when the text wraps onto more than one line: at least 1.5 times the font size. (SC 1.4.12)

Automation: whether the text wraps is an estimate, since text measurement is approximate.

Overlaps: C35 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/78fd32/>

#### DON'T

```html
<p style="line-height: 1em !important; max-width: 200px;">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

#### DO

```html
<p style="line-height: 2em !important; max-width: 200px;">
  The toy brought back fond memories of being lost in the rain forest.
</p>
```

---

### b33eff - Don't lock the page orientation with CSS rotation in orientation media queries

Don't rotate an element by 90 degrees (with `rotate` or a rotating `transform`) inside an `orientation: portrait` or `orientation: landscape` media query; a net rotation of a whole turn passes. (SC 1.3.4)

Automation: the renderer lays out one viewport, so orientation media queries have to be scanned in the stylesheet, as G195 does for `:focus`.

Overlaps: F97 in wcag2.2-rules.md

Source: <https://www.w3.org/WAI/standards-guidelines/act/rules/b33eff/>

#### DON'T

```html
<style>
  @media (orientation: portrait) {
    html {
      transform: rotate(1.5708rad);
      width: min(100vw, 100vh);
      height: min(100vw, 100vh);
    }
  }
</style>
```

#### DO

```html
<style>
  @media (orientation: portrait) {
    html {
      transform: rotateZ(1turn);
    }
  }
</style>
```

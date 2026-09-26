# ARIA in HTML Accessibility Checker Rules

This document lists accessibility-checker rules taken from the [ARIA in HTML](https://www.w3.org/TR/html-aria/) W3C Recommendation of 11 August 2026, as published in the [w3c/html-aria](https://github.com/w3c/html-aria) repository at commit `e277aa3e8b491aadc363372253b6fc2176f7b47c` (tag `REC-html-aria-20260811`). The spec has no rule IDs of its own, so each rule uses a custom `HTMLARIA` prefix and a three-digit number, in document order.

The list covers every author requirement in the spec, including those that can't be tested automatically, since it is also meant for AI review. Requirements aimed at user agents or conformance checkers are left out, as are lists the spec delegates to [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/) and [HTML-AAM](https://www.w3.org/TR/html-aam-1.0/) (supported attributes per role, abstract roles, roles that allow naming), which a checker has to take from those specifications.

An `Automation:` line says what part of a rule can't be checked from a static render, an `Overlaps:` line names an entry in `wcag2.2-rules.md` covering the same markup, and a `Prevailing rule:` line names the rule that wins when two conflict (see [overlap.md](./overlap.md)).

## Table of Contents

1. [Roles](#roles)
2. [ARIA Attributes](#aria-attributes)
3. [Native HTML Attribute Equivalents](#native-html-attribute-equivalents)
4. [Deprecated Features](#deprecated-features)
5. [Syntax](#syntax)
6. [Content Model](#content-model)

---

## Roles

### HTMLARIA001 - Only use roles allowed on the element

Authors MUST NOT set a `role` the table below doesn't allow on the element. "Any" allows every role (the implicit role and `generic` are discouraged, see HTMLARIA002 and HTMLARIA003); "None" allows no role other than the implicit one, which is NOT RECOMMENDED. ([Document conformance requirements](https://www.w3.org/TR/html-aria/#docconformance); MUST NOT)

| Element | Implicit role | Roles authors may set |
|---|---|---|
| `a` with `href` | `link` | `button`, `checkbox`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `switch`, `tab`, `treeitem`; DPub `doc-backlink`, `doc-biblioref`, `doc-glossref`, `doc-noteref` |
| `a` without `href`, `b`, `bdi`, `bdo`, `data`, `i`, `pre`, `q`, `samp`, `small`, `span`, `u` | `generic` | Any |
| `abbr`, `canvas`, `cite`, `kbd`, `mark`, `rp`, `rt`, `ruby`, `var` | none | Any |
| `address`, `hgroup` | `group` | Any |
| `area` with `href` | `link` | None |
| `area` without `href` | `generic` | `button`, `link` |
| `article` | `article` | `application`, `document`, `feed`, `main`, `none`, `presentation`, `region` |
| `aside` | `complementary` | `feed`, `none`, `note`, `presentation`, `region`, `search`; DPub `doc-dedication`, `doc-example`, `doc-footnote`, `doc-glossary`, `doc-pullquote`, `doc-tip` |
| `audio`, `video` | none | `application` |
| autonomous custom element | from `ElementInternals`, otherwise `generic` | None if `ElementInternals` defines a role, otherwise Any |
| form-associated custom element | from `ElementInternals`, otherwise `generic` | None if `ElementInternals` defines a role, otherwise `button`, `checkbox`, `combobox`, `listbox`, `progressbar`, `group`, `radio`, `radiogroup`, `searchbox`, `slider`, `spinbutton`, `switch`, `textbox` |
| `blockquote` | `blockquote` | Any |
| `body` | `generic` | None |
| `br`, `wbr` | none | `none`, `presentation` |
| `button` | `button` | `checkbox`, `combobox`, `gridcell`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `separator`, `slider`, `switch`, `tab`, `treeitem`; None if it is the first child of a `select` |
| `caption` | `caption` | None |
| `code` | `code` | Any |
| `datalist` | `listbox` | None |
| `dd`, `legend`, `picture` | none | None |
| `del`, `s` | `deletion` | Any |
| `details`, `optgroup` | `group` | None |
| `dfn` | `term` | Any |
| `dialog` | `dialog` | `alertdialog` |
| `div` | `generic` | `none`, `presentation` if a direct child of `dl`, otherwise Any |
| `dl` | none | `group`, `list`, `none`, `presentation` |
| `dt` | none | `listitem` |
| `em` | `emphasis` | Any |
| `embed`, `iframe` | none | `application`, `document`, `img`, `image`, `none`, `presentation` |
| `fieldset` | `group` | `none`, `presentation`, `radiogroup` |
| `figcaption` | none | `group`, `none`, `presentation` |
| `figure` | `figure` | DPub `doc-example` if it has a `figcaption` descendant, otherwise Any |
| `footer` | `contentinfo`, or `generic` inside `article`/`aside`/`main`/`nav`/`section` or their roles | `group`, `none`, `presentation`; DPub `doc-footnote` |
| `form` | `form` | `none`, `presentation`, `search` |
| `h1` to `h6` | `heading` | `none`, `presentation`, `tab`; DPub `doc-subtitle` |
| `header` | `banner`, or `generic` inside `article`/`aside`/`main`/`nav`/`section` or their roles | `group`, `none`, `presentation` |
| `hr` | `separator` | `none`, `presentation`; DPub `doc-pagebreak` |
| `html` | `generic` | None (`document` and `generic` are allowed but NOT RECOMMENDED) |
| `img` with an accessible name (e.g. non-empty `alt`) | `img` or `image` | `button`, `checkbox`, `link`, `math`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `meter`, `option`, `progressbar`, `radio`, `scrollbar`, `separator`, `slider`, `switch`, `tab`, `treeitem`; DPub `doc-cover` |
| `img` with no `alt` or no accessible name | `img` or `image` | `none`, `presentation` |
| `img` with `alt=""` and no other naming method | `none`/`presentation` | None |
| `input type=button` | `button` | `checkbox`, `combobox`, `gridcell`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `separator`, `slider`, `switch`, `tab`, `treeitem` |
| `input type=checkbox` | `checkbox` | `menuitemcheckbox`, `option`, `switch`; `button` if used with `aria-pressed` |
| `input type=color`, `date`, `datetime-local`, `file`, `month`, `password`, `time`, `week` | none | None |
| `input type=email`, `tel`, `url` without `list` | `textbox` | None |
| `input type=image` | `button` | `button`, `checkbox`, `gridcell`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `separator`, `slider`, `switch`, `tab`, `treeitem` (all NOT RECOMMENDED, see HTMLARIA005) |
| `input type=reset`, `submit` | `button` | the `input type=image` list plus `combobox` (all NOT RECOMMENDED, see HTMLARIA005) |
| `input type=number` | `spinbutton` | None |
| `input type=radio` | `radio` | `menuitemradio` |
| `input type=range` | `slider` | None |
| `input type=search` without `list` | `searchbox` | None |
| `input type=text`, or missing/invalid `type`, without `list` | `textbox` | `combobox`, `searchbox`, `spinbutton` |
| `input type=text`, `search`, `tel`, `url`, `email`, or missing/invalid `type`, with `list` | `combobox` | None |
| `ins` | `insertion` | Any |
| `label` | none | None if associated with a labelable element, otherwise Any |
| `li` | `listitem` if its parent is exposed as `list`, otherwise `generic` | None if its parent has an implicit or explicit `list` role, otherwise Any |
| `main` | `main` | None |
| `math` | `math` | None |
| `menu`, `ol`, `ul` | `list` | `group`, `listbox`, `menu`, `menubar`, `none`, `presentation`, `radiogroup`, `tablist`, `toolbar`, `tree` |
| `meter` | `meter` | None |
| `nav` | `navigation` | `menu`, `menubar`, `none`, `presentation`, `tablist`; DPub `doc-index`, `doc-pagelist`, `doc-toc` |
| `object` | none | `application`, `document`, `img`, `image` |
| `option` (in a list of options or a `datalist`) | `option` | None |
| `output` | `status` | Any |
| `p` | `paragraph` | Any |
| `progress` | `progressbar` | None |
| `search` | `search` | `form`, `group`, `none`, `presentation`, `region` |
| `section` | `region` if it has an accessible name, otherwise `generic` | `alert`, `alertdialog`, `application`, `banner`, `complementary`, `contentinfo`, `dialog`, `document`, `feed`, `group`, `log`, `main`, `marquee`, `navigation`, `none`, `note`, `presentation`, `search`, `status`, `tabpanel`; DPub `doc-abstract`, `doc-acknowledgments`, `doc-afterword`, `doc-appendix`, `doc-bibliography`, `doc-chapter`, `doc-colophon`, `doc-conclusion`, `doc-credit`, `doc-credits`, `doc-dedication`, `doc-endnotes`, `doc-epigraph`, `doc-epilogue`, `doc-errata`, `doc-example`, `doc-foreword`, `doc-glossary`, `doc-index`, `doc-introduction`, `doc-notice`, `doc-pagelist`, `doc-part`, `doc-preface`, `doc-prologue`, `doc-pullquote`, `doc-qna`, `doc-toc` |
| `select` (drop-down box) | `combobox` | `menu` |
| `select` (list box) | `listbox` | None |
| `selectedcontent` | `generic` | None inside a `select`, otherwise Any |
| `strong` | `strong` | Any |
| `sub` / `sup` | `subscript` / `superscript` | Any |
| `summary` | varies by user agent | None if it is the summary of its parent `details`, otherwise Any |
| `svg` | `graphics-document` | Any |
| `table` | `table` | Any |
| `tbody`, `tfoot`, `thead` | `rowgroup` | Any |
| `td` | `cell` or `gridcell`, from the ancestor `table`'s role | None if the ancestor `table` is exposed as `table`, `grid` or `treegrid`, otherwise Any |
| `th` | `columnheader`, `rowheader`, `cell` or `gridcell`, from the ancestor `table`'s role | None if the ancestor `table` is exposed as `table`, `grid` or `treegrid`, otherwise Any |
| `textarea` | `textbox` | None |
| `time` | `time` | Any |
| `tr` | `row` | None if the ancestor `table` has role `table`, `grid` or `treegrid`, otherwise Any |
| `base`, `col`, `colgroup`, `head`, `input type=hidden`, `link`, `map`, `meta`, `noscript`, `param`, `script`, `slot`, `source`, `style`, `template`, `title`, `track` | none | None (no `aria-*` attributes either, see HTMLARIA007) |

Automation: partial. A custom element's role set through `ElementInternals` comes from script, so custom elements need AI review.

Overlaps: F92 in wcag2.2-rules.md (this table allows `role=presentation` on `h1`-`h6`, `table` and lists; F92 still fails it when the element conveys structure)

Prevailing rule: F92 in wcag2.2-rules.md (see C5 in [overlap.md](./overlap.md#conflicts)). A role allowed by this table is still a failure when `none`/`presentation` removes structure the content conveys (e.g. a data table).

#### DON'T

```html
<button role="heading">search</button>
```

#### DO

```html
<h2>search</h2>
```

---

### HTMLARIA002 - Don't set a role or aria-* value the element already has implicitly

Don't set a `role` or `aria-*` value matching the element's implicit semantics (the "Implicit role" column of HTMLARIA001, or an `aria-level` on `h1`-`h6` equal to its number). `role=list` on a `ul` whose markers are removed is an accepted exception. ([Author requirements for use of ARIA in HTML](https://www.w3.org/TR/html-aria/#rules-wd); NOT RECOMMENDED)

Automation: partial. Whether a redundant role is justified (e.g. a `ul` without markers) needs AI review.

#### DON'T

```html
<button role="button">...</button>
<details>
  <summary role="button">more information</summary>
  ...
</details>
```

#### DO

```html
<button>...</button>
<details>
  <summary>more information</summary>
  ...
</details>
```

---

### HTMLARIA003 - Don't use the generic role

Don't set `role="generic"` on any element, nor `generic` or `document` on `html`. Use a `div`, or `role="none"`/`role="presentation"`, instead. ([Document conformance requirements](https://www.w3.org/TR/html-aria/#docconformance); SHOULD NOT)

#### DON'T

```html
<article role="generic">...</article>
```

#### DO

```html
<div>...</div>
```

---

### HTMLARIA004 - Don't use abstract ARIA roles

Don't set an abstract role (listed in [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/#isAbstract)), such as `role="select"`, on any element. ([Conformance checking requirements](https://www.w3.org/TR/html-aria/#conformance); MUST NOT)

#### DON'T

```html
<div role="select">...</div>
```

#### DO

```html
<div role="combobox">...</div>
```

---

### HTMLARIA005 - Use a button element rather than re-roling input type=image, reset or submit

Don't set any `role` on `input type=image`, `reset` or `submit`; use an element that allows the role, such as `button`. ([`input type=submit`](https://www.w3.org/TR/html-aria/#el-input-submit); NOT RECOMMENDED)

Prevailing rule: ARIA-USAGE001 in wai-aria-1.2-rules.md (see C6 in [overlap.md](./overlap.md#conflicts)). The `button role="switch"` DO example shows the allowed role, not the recommended control; prefer a native `input type="checkbox"` where it fits.

#### DON'T

```html
<input type="submit" role="switch" value="Notifications">
```

#### DO

```html
<button type="button" role="switch" aria-checked="false">Notifications</button>
```

---

### HTMLARIA006 - Test DPub ARIA roles with users when using them outside digital publishing

*Informative.* Avoid `doc-*` roles outside digital publishing; where they are used on a website, test them with users. ([DPub usage note](https://www.w3.org/TR/html-aria/#dpub-usage-note))

Automation: none. Whether a page is publishing content, or was tested with users, needs AI review.

#### DON'T

```html
<!-- product page on a shop website -->
<aside role="doc-tip">Free returns within 30 days.</aside>
```

#### DO

```html
<aside>Free returns within 30 days.</aside>
```

---

## ARIA Attributes

### HTMLARIA007 - Only use aria-* attributes allowed on the element

Only use global `aria-*` attributes and those supported by the element's role (explicit, else implicit, as defined in [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/#global_states)), and never ones that role prohibits; the elements below are more restricted. ([Document conformance requirements](https://www.w3.org/TR/html-aria/#docconformance); MUST NOT)

| Element | Allowed `aria-*` attributes |
|---|---|
| `base`, `col`, `colgroup`, `datalist`, `head`, `html`, `input type=hidden`, `link`, `map`, `meta`, `noscript`, `param`, `script`, `slot`, `source`, `style`, `template`, `title`, `track`; a `button` that is the first child of a `select`; a `selectedcontent` inside a `select` | none |
| `br`, `wbr`, `picture` | `aria-hidden` only |
| `img` with `alt=""` and no other naming method | `aria-hidden="true"` only |
| `caption`, `legend`, `label` | global only |
| `body` | global attributes allowed for `generic`, except `aria-hidden="true"` (see HTMLARIA009) |
| `input type=color` | global and `aria-disabled` |
| `input type=file` | global, `aria-disabled`, `aria-invalid` and `aria-required` |
| `summary` that is the summary of its parent `details` | global, `aria-disabled` and `aria-haspopup` |
| `dd` | global and those of the `definition` role |
| `input type=date`, `datetime-local`, `month`, `password`, `time`, `week` | global and those of the `textbox` role |
| every other element | global and those of its allowed roles |

#### DON'T

```html
<br aria-label="line break">
<input type="file" aria-valuenow="3">
```

#### DO

```html
<br aria-hidden="true">
<input type="file" aria-required="true">
```

---

### HTMLARIA008 - Don't name elements whose role prohibits naming

Don't set `aria-label` or `aria-labelledby` on a Naming Prohibited element unless an allowed explicit role that permits naming ([WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/#namefromprohibited)) overrides its implicit role. ([Requirements for use of ARIA attributes to name elements](https://www.w3.org/TR/html-aria/#docconformance-naming); MUST NOT)

Naming Prohibited elements:

- `a` without `href`, `abbr`, `area` without `href`, `b`, `bdi`, `bdo`, `body`, `caption`, `cite`, `code`, `data`, `del`, `div`, `em`, `figcaption`, `i`, `ins`, `kbd`, `legend`, `mark`, `p`, `pre`, `q`, `rp`, `rt`, `s`, `samp`, `selectedcontent`, `small`, `span`, `strong`, `sub`, `sup`, `time`, `u`, `var`
- `footer`, `header` and custom elements exposed as `generic`
- `label` and autonomous custom elements exposed as `generic` or another role that prohibits naming

Automation: partial. A custom element's role set through `ElementInternals` comes from script, so custom elements need AI review.

Prevailing rule: ARIA-USAGE001 in wai-aria-1.2-rules.md (see C6 in [overlap.md](./overlap.md#conflicts)). The `p role="link"` and `span role="button"` DO examples show how naming becomes allowed, not the recommended controls; prefer native `a href` and `button` elements.

#### DON'T

```html
<p aria-label="...">...</p>
<span aria-label="...">...</span>
<div aria-labelledby="...">...</div>
```

#### DO

```html
<p role="link" tabindex="0" aria-label="...">...</p>
<span role="button" tabindex="0" aria-label="...">...</span>
<div role="article" aria-labelledby="...">...</div>
```

---

### HTMLARIA009 - Never put aria-hidden on the body element or on focusable elements

Don't set `aria-hidden` on `body` or on any focusable element, including one with `tabindex="-1"`. ([`hidden` attribute](https://www.w3.org/TR/html-aria/#att-hidden); MUST NOT)

Automation: partial. Elements made focusable by script at runtime need AI review.

#### DON'T

```html
<body aria-hidden="true">
<button aria-hidden="true">Close</button>
<div tabindex="-1" aria-hidden="true">...</div>
```

#### DO

```html
<body>
<button>Close</button>
<div tabindex="-1">...</div>
```

---

### HTMLARIA010 - Don't add aria-hidden="true" to an element that already has the hidden attribute

Don't set `aria-hidden="true"` on an element with `hidden` (NOT RECOMMENDED), and never on one with `hidden="until-found"` (MUST NOT). ([`hidden` attribute](https://www.w3.org/TR/html-aria/#att-hidden))

#### DON'T

```html
<div hidden="until-found" aria-hidden="true">...</div>
```

#### DO

```html
<div hidden="until-found">...</div>
```

---

### HTMLARIA011 - Don't use aria-* attributes the element's own row advises against

Don't use these attributes on the native elements that handle them. ([Document conformance requirements](https://www.w3.org/TR/html-aria/#docconformance); SHOULD NOT)

- `aria-disabled="true"` on `a` with `href` (NOT RECOMMENDED; remove `href` instead)
- `aria-haspopup` on `input type=text`, `search`, `tel`, `url`, `email` (or missing/invalid `type`) with `list`
- `aria-selected` on `option`
- `aria-multiselectable` on `select`

#### DON'T

```html
<a href="/archive" aria-disabled="true">Archive</a>
<select aria-multiselectable="true">...</select>
```

#### DO

```html
<a role="link" aria-disabled="true">Archive</a>
<select multiple>...</select>
```

---

## Native HTML Attribute Equivalents

### HTMLARIA012 - Build a custom ARIA widget rather than add ARIA states the native element doesn't support

*Informative.* Don't add an ARIA state that HTML has no equivalent for on the native element (e.g. `aria-checked` on `option`, `aria-disabled` on `a href`); build a custom ARIA widget instead. ("ARIA semantics that extend and diverge from HTML")

Automation: partial. Whether a custom widget serves the intent better needs AI review.

Prevailing rule: ARIA-USAGE001 in wai-aria-1.2-rules.md (see C6 in [overlap.md](./overlap.md#conflicts)). A custom widget is the fallback when the native element can't express the state, not the default; first check whether a native control (e.g. `input type="checkbox"`) fits.

#### DON'T

```html
<select>
  <option aria-checked="true">Bold</option>
</select>
```

#### DO

```html
<div role="listbox" aria-label="Formatting">
  <div role="option" aria-checked="true">Bold</div>
</div>
```

---

### HTMLARIA013 - Don't pair a native HTML attribute with a conflicting aria-* attribute

Don't combine these native attributes with the `aria-*` attribute listed; use the native attribute (or the `indeterminate` IDL attribute for "mixed") instead. ([Requirements for use of ARIA attributes in place of equivalent HTML attributes](https://www.w3.org/TR/html-aria/#docconformance-attr); MUST NOT)

| Element or native attribute | Don't use |
|---|---|
| any element that allows `checked` (`input type=checkbox`, `input type=radio`) | `aria-checked` |
| `disabled` (including on `option` and `optgroup`) | `aria-disabled="false"` |
| `required` (`input`, `textarea`, `select`) | `aria-required="false"` |
| `readonly` (`input`, `textarea`, form-associated custom elements) | `aria-readonly="false"` |
| `contenteditable="true"`, set or inherited | `aria-readonly="true"` |
| `placeholder` | `aria-placeholder` |
| `max` (`meter`, `progress`, `input`) | `aria-valuemax` |
| `min` (`meter`, `input`) | `aria-valuemin` |
| `colspan` (`td`, `th`) | `aria-colspan` with a different value |
| `rowspan` (`td`, `th`) | `aria-rowspan` with a different value |

#### DON'T

```html
<input type="checkbox" checked aria-checked="false">
<input type="text" required aria-required="false">
<input type="text" placeholder="Search" aria-placeholder="Search">
```

#### DO

```html
<input type="checkbox" checked>
<input type="text" required>
<input type="text" placeholder="Search">
```

---

### HTMLARIA014 - Don't repeat a native HTML attribute with its aria-* counterpart

Don't set both a native attribute and its `aria-*` equivalent, even with matching values. ([Requirements for use of ARIA attributes in place of equivalent HTML attributes](https://www.w3.org/TR/html-aria/#docconformance-attr); SHOULD NOT)

- `aria-disabled="true"` with `disabled`
- `aria-required="true"` with `required`
- `aria-readonly="true"` with `readonly`
- `aria-colspan` with `colspan`, `aria-rowspan` with `rowspan`
- `aria-valuemax`/`aria-valuemin` on `meter`, `input` or `progress` (`max` only), even without `max`/`min`

Overlaps: H90 in wcag2.2-rules.md (its DO example uses `required aria-required="true"`, which this rule advises against)

#### DON'T

```html
<button disabled aria-disabled="true">Save</button>
<progress value="30" max="100" aria-valuemax="100"></progress>
```

#### DO

```html
<button disabled>Save</button>
<progress value="30" max="100"></progress>
```

---

## Deprecated Features

### HTMLARIA015 - Don't use deprecated ARIA roles or attributes

Don't use deprecated roles or attributes on any element. ([Requirements for deprecated ARIA role, state and property attributes](https://www.w3.org/TR/html-aria/#docconformance-deprecated); SHOULD NOT)

- `role="directory"`: use a native list or `role="list"`
- `role="doc-biblioentry"`, `role="doc-endnote"`: use a plain `li`
- `aria-dropeffect`, `aria-grabbed`: no replacement

#### DON'T

```html
<ul role="directory">...</ul>
<ol>
  <li role="doc-endnote">...</li>
</ol>
```

#### DO

```html
<ul>...</ul>
<ol>
  <li>...</li>
</ol>
```

---

## Syntax

### HTMLARIA016 - Write role and ARIA token values in lowercase

Write `role` tokens and token values of `aria-*` attributes in ASCII lowercase. ([Case requirements](https://www.w3.org/TR/html-aria/#case-sensitivity); SHOULD)

Automation: partial. The spec's advice to test with browsers and assistive technologies needs manual or AI review.

#### DON'T

```html
<div role="MAIN">...</div>
<a href="home/" aria-current="Page">home</a>
```

#### DO

```html
<div role="main">...</div>
<a href="home/" aria-current="page">home</a>
```

---

## Content Model

### HTMLARIA017 - Don't nest interactive content inside elements with an interactive role

*Informative.* Don't put interactive content or elements with `tabindex` inside roles `button`, `checkbox`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `switch` or `tab`, nor interactive content inside `img`/`image`. Also avoid `main` inside roles such as `article`, `navigation` or `textbox`, `form` inside `role=form`, `meter` inside `role=meter` and `progress` inside `role=progressbar`. ("Allowed descendants of ARIA roles")

#### DON'T

```html
<button>
  <div role="button">...</div>
</button>
<div role="button">
  <button>...</button>
</div>
<div role="link">
  <textarea>...</textarea>
</div>
```

#### DO

```html
<div role="button" tabindex="0">...</div>
<button>...</button>
<div role="link" tabindex="0">...</div>
<textarea>...</textarea>
```

---

### HTMLARIA018 - Don't rely on a role to make invalid HTML nesting work

*Informative.* Don't place an element where HTML's content model forbids it, even with a `role` (e.g. a `div role=link` inside a `p`); use an element allowed there, such as `span`. ("Adhere to the rules of HTML")

Automation: none. The HTML parser repairs the nesting before the DOM exists, so it needs source inspection or AI review.

#### DON'T

```html
<p>
  ... <div role=link tabindex=0>...</div> ...
</p>
```

#### DO

```html
<p>
  ... <span role=link tabindex=0>...</span> ...
</p>
```

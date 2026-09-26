# Overlaps and Conflicts Between Rule Files

This document compares the four rule lists and records where they check the same thing (duplicates) and where they disagree (conflicts):

- [wcag2.2-rules.md](./wcag2.2-rules.md): WCAG 2.2 techniques and failures (IDs such as `H37`, `F68`, `TGT001`)
- [act-rules.md](./act-rules.md): W3C-approved ACT rules (6-character IDs such as `23a2a8`)
- [wai-aria-1.2-rules.md](./wai-aria-1.2-rules.md): WAI-ARIA 1.2 author requirements (`ARIA-*` IDs)
- [html-aria-rules.md](./html-aria-rules.md): ARIA in HTML author requirements (`HTMLARIA*` IDs)

It covers every rule in the four files, including the ones added for AI review, which have no `Overlaps:` line in their own file yet. A duplicate is listed when two rules would flag the same markup for the same reason, even when one is stricter or broader. A conflict is listed when following one rule, or copying its DO example, breaks another rule. Each conflict ends with a decision on which rule prevails, and every rule that loses has a `Prevailing rule:` line in its own file naming the rule that wins and linking back here.

## Table of Contents

1. [Duplicates](#duplicates)
2. [Conflicts](#conflicts)
3. [Differences in strictness](#differences-in-strictness)
4. [Apparent conflicts that aren't](#apparent-conflicts-that-arent)
5. [Rules with no counterpart](#rules-with-no-counterpart)

---

## Duplicates

Rules on the same row check the same thing. "Scope" notes where they differ.

### Page, language and images

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| Page has a non-empty title | H25 | 2779a5 | | | ACT counts only the first `title`, even one in `body`. |
| Title describes the page | F25 | c4a8a4 | | | Both are judgements, for AI review. |
| `html` has a `lang` | H57 | b5c3f8, bf051a | | | ACT splits the check in two: `b5c3f8` requires a `lang` that isn't empty, and `bf051a` requires a valid primary language subtag from the IANA registry. |
| Image has a text alternative | H37, F65 | 23a2a8 | ARIA-NAME001 (`img` role) | | ACT also accepts `role="none"`/`presentation`. ARIA covers `role="img"` containers. |
| Alt text serves the image's purpose | F30 | qt1vmo | | | F30 lists placeholder text such as "image" or a filename. `qt1vmo` is a general judgement. |
| Decorative images stay decorative | H67 | 46ca7f | ARIA-PRES001, ARIA-PRES002 | HTMLARIA007 (`img alt=""` allows only `aria-hidden="true"`) | H67 also forbids `title`. `46ca7f` and PRES001 flag the conflicts that re-expose the image (a global ARIA attribute, or focusability). |
| Image button has a name | H36 | 59796f | | | ACT also fails the browser default name ("Submit Query"). |
| Image map `area` has a name | H24 | c487ae (covers `area[href]`) | | | |
| SVG image has a name | | 7d6734 | ARIA-NAME001 (`img` requires a name) | | |

### Names, labels and forms

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| Controls have an accessible name | F68, ARIA14, ARIA16 | 97a4e1 (button), m6b1q3 (menuitem), 2t702h (summary), e086e5 (form fields) | ARIA-NAME001 | | NAME001 covers every role that requires a name, including non-controls (`dialog`, `table`, `region`, ...). |
| Form fields have a label | H44 | e086e5 | ARIA-NAME001 | | H44 requires a `label` element. `e086e5` accepts any name. |
| Links have a name | H30 (partly), F89 | c487ae | ARIA-NAME001 | | `c487ae` only checks that a name exists. H30 also judges the text. |
| Valid `autocomplete` values | H98, F107 | 73f2c2 | | | ACT defines the full token grammar. |
| Choosing between `aria-label` and `aria-labelledby` | ARIA14, ARIA16 | | ARIA-USAGE003 | | |
| Errors described in text | G83 | | ARIA-STATE004, ARIA-IDREF004 | | ARIA adds rules on when to set `aria-invalid` and how to show or hide `aria-errormessage`. |

### Roles and ARIA attributes

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| Role value is a real, non-abstract role | | 674b10 | ARIA-ROLE001, ARIA-ROLE002 | HTMLARIA004 | ACT passes when at least one token is valid. ROLE001 also warns about invalid tokens placed before it. |
| Don't use `role="generic"` | | | ARIA-ROLE003 | HTMLARIA003 | |
| Role is allowed on this HTML element | F92 (partly) | | | HTMLARIA001, HTMLARIA005 | |
| `aria-*` name is defined | | 5f99a7 | ARIA-ATTR001 | HTMLARIA007 (implicitly) | ACT accepts the Graphics and DPub modules. ATTR001 lists only the 48 attributes of 1.2. |
| `aria-*` value has the right type | | 6a7281 | ARIA-VAL001 | | ACT doesn't require ID references to resolve. VAL001 sends that check to ARIA-IDREF001. |
| Required states and properties are set | ARIA5 (partly) | 4e8ab6 | ARIA-ATTR002 | | See conflict C2. |
| Attribute is supported (not prohibited) on the role | | | ARIA-ATTR003, ARIA-ATTR004, ARIA-DEPR003 | HTMLARIA007 | HTMLARIA007 adds limits per HTML element. |
| Don't name elements whose role prohibits a name | | | ARIA-ATTR004 | HTMLARIA008 | See strictness difference S4. |
| Role placed in its required parent, with its required children | | | ARIA-STRUCT001, ARIA-STRUCT002 | | |
| `aria-colspan`/`rowspan` in a native table | | | ARIA-ATTR005 | HTMLARIA013, HTMLARIA014 | |
| `aria-disabled` on a link with `href` | | | ARIA-STATE003 | HTMLARIA011, HTMLARIA012 | All three have the same DON'T example. |
| Deprecated `directory`, `aria-grabbed`, `aria-dropeffect` | | | ARIA-DEPR001, ARIA-DEPR002 | HTMLARIA015 | HTMLARIA015 also covers `doc-biblioentry` and `doc-endnote`. |
| Prefer native elements over ARIA widgets | F59, G202 | | ARIA-USAGE001 | HTMLARIA012 | See conflict C6. |

### Hidden and presentational content

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| No focusable content under `aria-hidden="true"` | | 6cfa84 | ARIA-HIDDEN001 | HTMLARIA009 | See strictness difference S1. |
| No focusable content inside roles whose children are presentational | | 307n5z | ARIA-PRES003 | HTMLARIA017 | See strictness difference S2. |
| `role="presentation"` on a focusable element or one with global ARIA attributes | | 46ca7f (images only) | ARIA-PRES001 | | |
| `role="presentation"` on structural content | F92 | | ARIA-PRES001 (partly) | HTMLARIA001 | See conflict C5. |

### ID references

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| ID references resolve | ARIA1, ARIA16 | | ARIA-IDREF001 | | |
| `id`s are unique | IDS001 | | ARIA-IDREF001 (mentions it) | | ACT's duplicate-id rule is deprecated, so it isn't listed. |
| `headers` refers to cells in the same table | F90 | a25f45 | | | |
| Tooltip referenced by `aria-describedby` | ARIA1 | | ARIA-IDREF005 | | |

### Keyboard, focus and dynamic behavior

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| Visible focus indicator | G195 | oj04fd | ARIA-FOCUS003 (focus shown on the `aria-activedescendant` target) | | G195 scans stylesheets. `oj04fd` compares pixels. |
| Everything works from the keyboard | G202 | akn7bn, 0ssw9k | ARIA-FOCUS001 | | ACT covers iframes and scrollable regions. FOCUS001 covers widgets. |
| States kept in sync with the UI | ARIA5 | | ARIA-STATE001, ARIA-WIDGET004 | | |
| Status messages exposed | F103 | | ARIA-LIVE001, ARIA-LIVE002 | | |
| Modal dialogs | G21 | | ARIA-FOCUS006 | | See [apparent conflicts](#apparent-conflicts-that-arent). |
| Keyboard shortcuts | F99 | | ARIA-ATTR007, ARIA-FOCUS007 | | See strictness difference S5. |

### Visual presentation

| Topic | wcag2.2 | ACT | WAI-ARIA | ARIA in HTML | Scope |
|---|---|---|---|---|---|
| Text contrast (AA) | G18, G145 | afw4f7 | | | |
| Text contrast (AAA) | G18, G145 (lower threshold) | 09o5cg | | | See strictness difference S3. |
| Timed meta refresh | F40 / F41 | bc659a, bisz58 | | | See strictness difference S3. |
| Text spacing overrides | C35 | 24afc2, 9e45ec, 78fd32 | | | ACT only checks `!important` values in `style` attributes. |
| Locked orientation | F97 | b33eff | | | |

---

## Conflicts

These are cases where following one rule, or copying its DO example, breaks another rule.

Decisions follow these principles, in order:

1. **WCAG decides pass or fail.** It is what the laws point to (EN 301 549, Section 508, RGAA). ACT rules are W3C's approved way of testing WCAG, so when an ACT rule and a WCAG technique differ on the test, the ACT rule is more precise.
2. **ARIA in HTML governs native HTML elements; WAI-ARIA 1.2 governs explicit `role` attributes.** ARIA in HTML is the newer spec (2026) and was written specifically for HTML. Applying WAI-ARIA requirements only to explicit roles resolves C4, C10 and C11 on its own.
3. **A MUST beats a SHOULD, and normative text beats notes and informative sections.**
4. **Examples are illustrations, not rules.** When an example breaks another rule, the example is what changes.

Only C2 and C3 are genuine disagreements between standards, and in both the newer, more specific source wins. The rest are resolved by fixing an example (C1, C6, C8, C12), by scoping a rule (C4, C7, C9, C10, C11), or by applying both rules together (C5).

### C1 - `required` together with `aria-required="true"`

- **H90** DO: `<input id="lastname" name="lastname" required aria-required="true">`
- **HTMLARIA014**: authors SHOULD NOT set `aria-required="true"` on an element with `required`.

The H90 example fails HTMLARIA014 as written.

**Decision:** HTMLARIA014 prevails. It is a direct statement about native attributes, and H90 only asks that "required" be indicated, which `required` alone does. H90's DO example should drop `aria-required="true"`.

### C2 - Is `aria-selected` required on `option` and `treeitem`?

- **ARIA-ATTR002** lists `aria-selected` as required on `option`, and on `treeitem` (inherited from `option`). The agent that wrote it flagged the `treeitem` case as uncertain.
- **4e8ab6** says `option` doesn't need `aria-selected`, because WAI-ARIA 1.2 gives it a default value of `false`. Its passing example is `<li role="option">Zebra</li>` with no `aria-selected`.

A bare `role="option"` passes the ACT rule and fails ARIA-ATTR002. The two need to be reconciled against the spec before either is implemented.

**Decision:** 4e8ab6 prevails. It is W3C's approved reading of the spec: a required attribute with a default value doesn't have to be written, and WAI-ARIA 1.3 drops the requirement. ARIA-ATTR002 should skip required attributes that have a default. Check this against the 1.2 text before implementing it.

### C3 - The `image` role

- **HTMLARIA001** (ARIA in HTML, 2026) accepts `image` as the preferred synonym for `img`. The `img` element's implicit role is "`img` or `image`", and `embed`, `iframe` and `object` may take `role="image"`.
- **ARIA-ROLE001** (WAI-ARIA 1.2) has no `image` role, so it reports `role="image"` as unknown. The name comes from WAI-ARIA 1.3.

A page that follows ARIA in HTML fails ARIA-ROLE001. **674b10** accepts roles from "the WAI-ARIA specifications", so what it does depends on which ARIA version the checker uses.

**Decision:** HTMLARIA001 prevails. ARIA in HTML is newer, specific to HTML, and browsers already support `image`. ARIA-ROLE001 accepts the WAI-ARIA 1.2 roles plus the `image` synonym, not every 1.3 role.

### C4 - Tables without an accessible name

- **ARIA-NAME001** says `table` requires an author-provided name (from `aria-label`, `aria-labelledby` or a caption).
- The DO examples of **H51**, **H63**, **F91** and **a25f45** are `<table>` elements with no caption and no name.

Those examples fail ARIA-NAME001. More generally, many DO snippets in all four files leave out labels that aren't the point of the example (e.g. HTMLARIA013's `<input type="text" required>`). A checker run on the examples will find these.

**Decision:** WCAG prevails (H51, H63, F91, and a25f45 in act-rules.md). No WCAG criterion, ACT rule or major checker fails a native `<table>` for having no name; the WAI-ARIA requirement is about `role="table"`. ARIA-NAME001 applies to explicit roles only, and a name for native tables is a suggestion for AI review. Adding a `<caption>` to the WCAG examples is a free improvement.

### C5 - `role="presentation"` on tables, headings and lists

- **HTMLARIA001** allows `none`/`presentation` on `h1`-`h6`, `table`, `ul`/`ol`/`menu` and `dl`.
- **F92** fails `role="presentation"` on any element whose markup conveys structure, and its DON'T example is a data table with `role="presentation"`.
- **F46**'s DO example is a layout `<table role="presentation">`.

These fit together only if a checker can tell a data table from a layout table. The syntax check (HTMLARIA001) always passes the role, while F92 can fail the same markup.

**Decision:** F92 prevails, but both rules apply. HTMLARIA001 only says the role is *permitted* on the element; F92 decides whether using it is *correct*. F92 needs a data-table vs layout-table heuristic, with AI review for unclear cases.

### C6 - DO examples that use custom ARIA widgets

- **ARIA-USAGE001** says authors SHOULD use a native element when one exists. Its DON'T example is `<div role="checkbox" tabindex="0" aria-checked="false">`.
- The same markup appears as a DO example in:
  - **ARIA-ATTR002**: `<div role="checkbox" tabindex="0" aria-checked="false">`
  - **4e8ab6**: `<div role="checkbox" aria-checked="false" ...>`
  - **ARIA-ATTR006**: `<div role="checkbox" tabindex="0" aria-checked="mixed">`

  Other DO examples use custom widgets where a native element exists: **ARIA-STATE002** (`div role="radio"`), **HTMLARIA012** (a custom `listbox` rather than a native `select`), **HTMLARIA005** (`<button role="switch">`) and **HTMLARIA008** (`<p role="link">`, `<span role="button">`).

These DO examples show valid ARIA, not the recommended way to build the control. An AI reviewer given one rule's DO example could flag another rule's DO as a problem.

**Decision:** ARIA-USAGE001 prevails. "Use native elements first" is the first rule of ARIA, and every standard agrees with it. The DO examples of ARIA-ATTR002, 4e8ab6, ARIA-ATTR006, ARIA-STATE002, HTMLARIA005, HTMLARIA008 and HTMLARIA012 show valid ARIA, not the recommended control: change them to native elements, or label them "if a custom widget is unavoidable".

### C7 - `aria-disabled` on a link without `href`

- **HTMLARIA011** DO: `<a role="link" aria-disabled="true">Archive</a>`, the spec's way to show a disabled link.
- **ARIA-STATE003**: "authors are advised not to use `aria-disabled` on elements that cannot be disabled through features of the host language alone". Its DO example uses `<button disabled>` instead.

**Decision:** HTMLARIA011 prevails. `<a role="link" aria-disabled="true">` without `href` is the spec's own documented pattern, and ARIA-STATE003's "authors are advised" is a soft, general warning. Both agree that `href` + `aria-disabled` is wrong, so only that case is flagged.

### C8 - `aria-label` when the label text is visible

- **ARIA-USAGE003**: if the label text is in the DOM, authors SHOULD use `aria-labelledby` and SHOULD NOT use `aria-label`.
- **F96** DO: `<button aria-label="Go - search this site">Go</button>`. The visible text is "Go", but the name comes from `aria-label`.

**Decision:** ARIA-USAGE003 prevails on how to label; F96 still defines what is required (the visible text must be in the name). The extra context in F96's second DO example belongs in `aria-describedby`, not in an `aria-label`.

### C9 - Labelling a control that has no visible label

- **H65** recommends the `title` attribute when a visible `label` isn't possible (`<input type="search" title="Search the site">`).
- **ARIA-USAGE003**: when a visible label isn't possible, authors SHOULD use `aria-label`.

Neither rule forbids the other's technique. They recommend different fixes for the same finding.

**Decision:** ARIA-USAGE003 prevails as advice, and neither rule fails. `title` is shown only on mouse hover and assistive technology support for it is uneven, but H65 is still a WCAG-sufficient technique. `title` is never reported as an error; the help text recommends `aria-label`.

### C10 - Grouping radio buttons

- **H71** groups radio buttons with `fieldset`/`legend`, which HTML exposes as `group`.
- **ARIA-USAGE004**: radios that affect the same value SHOULD be in a `radiogroup`.

Native radios have the implicit role `radio`, so applying USAGE004 to them flags H71's DO example. HTMLARIA001 allows `<fieldset role="radiogroup">`, which satisfies both rules.

**Decision:** H71 prevails. A native `fieldset`/`legend` is enough, and ARIA-USAGE004 is meant for custom `role="radio"` widgets. ARIA-USAGE004 applies to explicit roles only; native radios are grouped by their `name` attribute.

### C11 - `aria-valuemin`/`aria-valuemax` on native range elements

- **ARIA-VAL004**: when the minimum and maximum are known, authors SHOULD set `aria-valuemin`/`aria-valuemax`.
- **HTMLARIA014**: `aria-valuemin`/`aria-valuemax` SHOULD NOT be used on `meter`, `progress` or `input`, even without `min`/`max`.

VAL004 has to apply only to elements with an explicit range role, never to native `meter`, `progress` or `input`.

**Decision:** HTMLARIA014 prevails. ARIA in HTML governs native elements, so ARIA-VAL004 applies to explicit range roles only.

### C12 - Assertive announcements in examples

- **ARIA-LIVE001**: SHOULD NOT use `aria-live="assertive"` or `role="alert"` unless the interruption is imperative.
- **ARIA-IDREF004** DO uses `aria-live="assertive"` for a form validation message. **G83** DO uses `role="alert"` for one.

Whether a validation error counts as imperative is a judgement call, but the examples teach the opposite default.

**Decision:** ARIA-LIVE001 prevails as a default, but nothing is reported: whether an announcement is urgent is a judgement call for AI review, and an error after a failed submit can reasonably be assertive. G83's DO example should use `aria-errormessage` or `role="status"`; ARIA-IDREF004's example is the spec's own and stays as written.

---

## Differences in strictness

These rules agree on intent but draw the line in different places. Implemented side by side, they produce findings on some markup and not others.

### S1 - Focusable content under `aria-hidden`

- **6cfa84** fails only elements in the *sequential* focus order. Its DO examples include `<input disabled aria-hidden="true" />`.
- **HTMLARIA009** forbids `aria-hidden` on *any* focusable element, including `tabindex="-1"`. Its DON'T example is `<div tabindex="-1" aria-hidden="true">`.
- **ARIA-HIDDEN001** also warns about `aria-hidden="false"` inside a hidden subtree.

`<div tabindex="-1" aria-hidden="true">` passes 6cfa84 and fails HTMLARIA009.

### S2 - Content inside roles with presentational children

- **307n5z** fails only descendants in the sequential focus order.
- **HTMLARIA017** fails any interactive descendant and any descendant with a `tabindex` attribute, even `-1`.
- **ARIA-PRES003** also fails descendants that have semantic roles, such as headings, lists or tables inside a `button`.

`<div role="button"><h3>Pro plan</h3></div>` fails PRES003 only.

### S3 - AA vs AAA thresholds

- **09o5cg** (contrast 7:1, or 4.5:1 for large text) and **bisz58** (meta refresh with no 20-hour exception) are the Level AAA versions of **afw4f7** and **bc659a**.
- **G18/G145** and **F40/F41** use the AA levels, which have lower contrast thresholds and allow a refresh after more than 20 hours.

A 72,001-second refresh passes F40/F41 and bc659a but fails bisz58. The contrast rules are spectrum rules with a `--*-threshold` flag, and CLAUDE.md asks for confirmation before adding a threshold or changing how thresholds work.

### S4 - Which elements can't be named

- **ARIA-ATTR004** prohibits `aria-label`/`aria-labelledby` on 11 roles: `caption`, `code`, `deletion`, `emphasis`, `generic`, `insertion`, `paragraph`, `presentation`/`none`, `strong`, `subscript` and `superscript`.
- **HTMLARIA008** also prohibits naming on elements with no matching role or with other roles, such as `abbr`, `cite`, `kbd`, `mark`, `time`, `legend`, `figcaption`, `rp`, `rt`, `samp`, `var`, and `label` when it is exposed as `generic`.

`<time aria-label="...">` fails HTMLARIA008 and passes ARIA-ATTR004.

### S5 - Single-key shortcuts

- **ARIA-ATTR007** gives `aria-keyshortcuts="A"` as a valid value (it only checks syntax).
- **F99** requires a single-character shortcut to be something users can turn off or remap, or that is active only while its control has focus.

`aria-keyshortcuts="A"` on a page-wide shortcut passes ATTR007 and is a candidate for F99.

### S6 - Unknown role tokens and letter case

- **674b10** passes `role="searchfield searchbox"`, since one token is valid. **ARIA-ROLE001** also warns about an invalid token placed before the first valid one.
- **ARIA-ROLE001** accepts `role="MAIN"`, since HTML role values are case-insensitive. **HTMLARIA016** says authors SHOULD write it in lowercase.

### S7 - `aria-description`

**ARIA-ATTR001** lists only the 1.2 attributes, so it reports `aria-description` (added in WAI-ARIA 1.3) as unknown. **5f99a7** accepts attributes defined in "the WAI-ARIA specifications", so whether it passes depends on the ARIA version the checker uses. Browsers already support `aria-description`, so this will come up on real sites.

---

## Apparent conflicts that aren't

- **G21 vs ARIA-FOCUS006.** G21 forbids keyboard traps. FOCUS006 keeps focus inside a modal dialog. G21's own DO example is a modal where Tab cycles inside the dialog and Escape closes it, which is what FOCUS006 asks for.
- **ARIA-ATTR002 on native elements.** ATTR002 requires `aria-checked` on `checkbox`, and `aria-controls`/`aria-expanded` on `combobox`. It also says the native equivalent satisfies the requirement. HTMLARIA013 forbids `aria-checked` on native checkboxes. There's no conflict as long as ATTR002 is checked only against explicit roles, as 4e8ab6 does.
- **HTMLARIA002 vs 4e8ab6.** HTMLARIA002 flags a redundant explicit role. 4e8ab6 skips explicit roles that match the implicit one. The two agree.
- **HTMLARIA011 vs ARIA-ATTR003 on `aria-selected`.** HTMLARIA011's SHOULD NOT applies to the native `option` element. ATTR003 supports `aria-selected` on the `option` *role*.

---

## Rules with no counterpart

These rules appear in only one file.

- **wcag2.2-rules.md:**
  - H64, H42, F43, G141, LNK001, G14/F24, G87, G93, G186, F93, F16, F54, F42, F55, F110, G1, G61, HLP001, G133, ANI001, F108, F101, F105, TGT001, RED001, AUT001, F109, C32, G142, F94, F95
  - H63 and H51: F91 and a25f45 cover some of the same table markup, but not the scope rule itself.
- **act-rules.md:**
  - de46e4 (`lang` on parts of the page, SC 3.1.2; wcag2.2 has no H58 entry)
  - 0va7u6 (images of text)
  - 8fc3b6 (`object` names)
  - b4f0c3 (viewport zoom)
- **wai-aria-1.2-rules.md:**
  - ARIA-ROLE004
  - ARIA-ATTR005 (only partly covered), ARIA-ATTR006, ARIA-ATTR008, ARIA-ATTR009
  - ARIA-VAL002 to ARIA-VAL005
  - ARIA-IDREF002 to ARIA-IDREF004
  - ARIA-STRUCT003, ARIA-STRUCT004
  - ARIA-WIDGET001 to ARIA-WIDGET007
  - ARIA-FOCUS002, ARIA-FOCUS004, ARIA-FOCUS005
  - ARIA-STATE002, ARIA-STATE005
  - ARIA-LMK001
  - ARIA-USAGE002, ARIA-USAGE004
- **html-aria-rules.md:**
  - HTMLARIA002, HTMLARIA006, HTMLARIA010, HTMLARIA016, HTMLARIA018

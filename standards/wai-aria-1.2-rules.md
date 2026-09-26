# WAI-ARIA 1.2 Accessibility Checker Rules

This document lists accessibility-checker rules derived from the author requirements of the [WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/) Recommendation (6 June 2023), taken from the [w3c/aria](https://github.com/w3c/aria) repository's `2023-06_REC` branch at commit `66caad8` (compiled September 2026).

Rule IDs follow the scheme `ARIA-<CATEGORY><NNN>`, where `<CATEGORY>` is one of `ROLE` Roles, `ATTR` States & Properties, `VAL` Attribute Values, `IDREF` ID References, `STRUCT` Required Context & Owned Elements, `NAME` Accessible Names, `PRES` Presentational Roles, `HIDDEN` Hidden Content, `WIDGET` Widgets, `FOCUS` Keyboard & Focus, `STATE` Dynamic States, `LIVE` Live Regions, `LMK` Landmarks, `USAGE` Semantic Choices, `DEPR` Deprecated Features, and `<NNN>` is a sequence number. Each entry cites the spec section it comes from.

Every author requirement (MUST, MUST NOT, SHOULD, SHOULD NOT) is listed, including those that can't be tested automatically and are left to AI review. Requirements aimed only at user agents, the IDL section, MAY statements and features that exist only in the 1.3 draft (e.g. `aria-description`) are left out. Implicit roles of HTML elements (e.g. `div` is `generic`) come from ARIA in HTML / HTML-AAM.

An `Automation:` line says what a static check can't cover, an `Overlaps:` line names matching entries in `wcag2.2-rules.md`, and a `Prevailing rule:` line says which rule wins a conflict recorded in [overlap.md](./overlap.md).

## Table of Contents

1. [Roles](#roles)
2. [States & Properties](#states--properties)
3. [Attribute Values](#attribute-values)
4. [ID References](#id-references)
5. [Required Context & Owned Elements](#required-context--owned-elements)
6. [Accessible Names](#accessible-names)
7. [Presentational Roles](#presentational-roles)
8. [Hidden Content](#hidden-content)
9. [Widgets](#widgets)
10. [Keyboard & Focus](#keyboard--focus)
11. [Dynamic States](#dynamic-states)
12. [Live Regions](#live-regions)
13. [Landmarks](#landmarks)
14. [Semantic Choices](#semantic-choices)
15. [Deprecated Features](#deprecated-features)

---

## Roles

### ARIA-ROLE001 - Use only role values defined in WAI-ARIA 1.2

Use a `role` whose value is one of the 82 non-abstract WAI-ARIA 1.2 roles below; an unknown or misspelled role does nothing, and case follows the host language. Put fallback tokens after the preferred one (e.g. `role="none presentation"`): it's an error when no token is valid, and a warning when an invalid token comes before the first valid one. DPUB-ARIA (`doc-*`) and Graphics-ARIA (`graphics-*`) roles are defined outside this spec. ([WAI-ARIA Roles](https://www.w3.org/TR/wai-aria-1.2/#introroles))

| Category | Roles |
|---|---|
| Widget | `button`, `checkbox`, `gridcell`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `progressbar`, `radio`, `scrollbar`, `searchbox`, `separator`, `slider`, `spinbutton`, `switch`, `tab`, `tabpanel`, `textbox`, `treeitem` |
| Composite widget | `combobox`, `grid`, `listbox`, `menu`, `menubar`, `radiogroup`, `tablist`, `tree`, `treegrid` |
| Document structure | `application`, `article`, `blockquote`, `caption`, `cell`, `code`, `columnheader`, `definition`, `deletion`, `directory`, `document`, `emphasis`, `feed`, `figure`, `generic`, `group`, `heading`, `img`, `insertion`, `list`, `listitem`, `math`, `meter`, `none`, `note`, `paragraph`, `presentation`, `row`, `rowgroup`, `rowheader`, `separator`, `strong`, `subscript`, `superscript`, `table`, `term`, `time`, `toolbar`, `tooltip` |
| Landmark | `banner`, `complementary`, `contentinfo`, `form`, `main`, `navigation`, `region`, `search` |
| Live region | `alert`, `log`, `marquee`, `status`, `timer` |
| Window | `alertdialog`, `dialog` |

Prevailing rule: HTMLARIA001 in html-aria-rules.md (see C3 in [overlap.md](./overlap.md#conflicts)). In HTML documents, `image` is accepted as a synonym of `img`, even though it is not a WAI-ARIA 1.2 role.

#### DON'T

```html
<div role="buton" tabindex="0">Save</div>
<table role="foo">...</table>
```

#### DO

```html
<div role="button" tabindex="0">Save</div>
<li role="none presentation">...</li>
```

---

### ARIA-ROLE002 - Never use abstract roles

Don't use abstract roles in content: `command`, `composite`, `input`, `landmark`, `range`, `roletype`, `section`, `sectionhead`, `select`, `structure`, `widget`, `window`. ([Abstract Roles](https://www.w3.org/TR/wai-aria-1.2/#isAbstract))

#### DON'T

```html
<div role="landmark" aria-label="Site links">...</div>
<div role="range" aria-valuenow="40">40%</div>
```

#### DO

```html
<div role="navigation" aria-label="Site links">...</div>
<div role="progressbar" aria-label="Upload" aria-valuenow="40">40%</div>
```

---

### ARIA-ROLE003 - Don't set role="generic" explicitly

Don't set `role="generic"`. Use `none`/`presentation` to remove semantics, or a role such as `group` to group and name content. ([generic](https://www.w3.org/TR/wai-aria-1.2/#generic))

#### DON'T

```html
<section role="generic" class="card">...</section>
```

#### DO

```html
<section role="none" class="card">...</section>
<!-- or, to group and name the content: -->
<div role="group" aria-label="Shipping options">...</div>
```

---

### ARIA-ROLE004 - Make static content inside role="application" reachable

Inside `role="application"`, make every non-decorative text and image reachable: reference it from a focusable element with `aria-labelledby`/`aria-describedby`, put it in a focusable `document` or `article`, or manage focus with `aria-activedescendant`. ([application](https://www.w3.org/TR/wai-aria-1.2/#application))

Automation: The `aria-activedescendant` technique needs JS execution and interaction.

#### DON'T

```html
<div role="application" aria-roledescription="Slide editor">
  <p>Use the arrow keys to move the selected shape.</p>
  ...
</div>
```

#### DO

```html
<div role="application" aria-roledescription="Slide editor" aria-describedby="editor-help" tabindex="0">
  <p id="editor-help">Use the arrow keys to move the selected shape.</p>
  ...
</div>
```

---

## States & Properties

### ARIA-ATTR001 - Use only aria-* attribute names defined in WAI-ARIA 1.2

Use only the 48 `aria-*` attributes defined in 1.2; any other name (e.g. the typo `aria-labeledby`) does nothing: `aria-activedescendant`, `aria-atomic`, `aria-autocomplete`, `aria-busy`, `aria-checked`, `aria-colcount`, `aria-colindex`, `aria-colspan`, `aria-controls`, `aria-current`, `aria-describedby`, `aria-details`, `aria-disabled`, `aria-dropeffect`, `aria-errormessage`, `aria-expanded`, `aria-flowto`, `aria-grabbed`, `aria-haspopup`, `aria-hidden`, `aria-invalid`, `aria-keyshortcuts`, `aria-label`, `aria-labelledby`, `aria-level`, `aria-live`, `aria-modal`, `aria-multiline`, `aria-multiselectable`, `aria-orientation`, `aria-owns`, `aria-placeholder`, `aria-posinset`, `aria-pressed`, `aria-readonly`, `aria-relevant`, `aria-required`, `aria-roledescription`, `aria-rowcount`, `aria-rowindex`, `aria-rowspan`, `aria-selected`, `aria-setsize`, `aria-sort`, `aria-valuemax`, `aria-valuemin`, `aria-valuenow`, `aria-valuetext`. ([State and Property Attributes](https://www.w3.org/TR/wai-aria-1.2/#host_general_attrs))

#### DON'T

```html
<input type="text" aria-labeledby="email-label">
<div aria-role="button" tabindex="0">Save</div>
```

#### DO

```html
<input type="text" aria-labelledby="email-label">
<div role="button" tabindex="0">Save</div>
```

---

### ARIA-ATTR002 - Provide every required state and property for the role

Give each role below a non-empty value for its required states and properties (subclass roles inherit them). A native equivalent, such as the checked state of `<input type="checkbox">` or the level of `<h2>`, satisfies the requirement. ([Required States and Properties](https://www.w3.org/TR/wai-aria-1.2/#requiredState))

| Role | Required |
|---|---|
| `checkbox`, `radio`, `switch`, `menuitemcheckbox`, `menuitemradio` (inherited from `menuitemcheckbox`) | `aria-checked` |
| `combobox` | `aria-controls`, `aria-expanded` |
| `heading` | `aria-level` |
| `meter`, `slider` | `aria-valuenow` |
| `option`, `treeitem` (inherited from `option`) | `aria-selected` |
| `scrollbar` | `aria-controls`, `aria-valuenow` |
| `separator` (only when focusable) | `aria-valuenow` |

Prevailing rule: 4e8ab6 in act-rules.md (see C2 in [overlap.md](./overlap.md#conflicts)). A required attribute with a default value (`aria-selected` on `option`, and on `treeitem` through inheritance) need not be set. Also, for the custom-widget DO example, ARIA-USAGE001 prevails (see C6 in [overlap.md](./overlap.md#conflicts)): prefer a native `input type="checkbox"`.

#### DON'T

```html
<div role="checkbox" tabindex="0">Subscribe to the newsletter</div>
<div role="heading">Billing address</div>
```

#### DO

```html
<div role="checkbox" tabindex="0" aria-checked="false">Subscribe to the newsletter</div>
<div role="heading" aria-level="2">Billing address</div>
```

---

### ARIA-ATTR003 - Use non-global states and properties only on roles that support them

Use non-global states and properties only on elements whose explicit or implicit role supports them (below, `*` = required). The global ones, allowed anywhere, are `aria-atomic`, `aria-busy`, `aria-controls`, `aria-current`, `aria-describedby`, `aria-details`, `aria-flowto`, `aria-hidden`, `aria-keyshortcuts`, `aria-label`, `aria-labelledby`, `aria-live`, `aria-owns`, `aria-relevant`, `aria-roledescription`, and the deprecated `aria-dropeffect` and `aria-grabbed`; `aria-disabled`, `aria-errormessage`, `aria-haspopup` and `aria-invalid` are covered by ARIA-DEPR003. ([Supported States and Properties](https://www.w3.org/TR/wai-aria-1.2/#supportedState))

| Attribute | Roles |
|---|---|
| `aria-activedescendant` | application, combobox, grid, group, listbox, menu, menubar, radiogroup, row, searchbox, spinbutton, tablist, textbox, toolbar, tree, treegrid |
| `aria-autocomplete` | combobox, searchbox, textbox |
| `aria-checked` | checkbox\*, menuitemcheckbox\*, menuitemradio\*, option, radio\*, switch\*, treeitem |
| `aria-colcount`, `aria-rowcount` | grid, table, treegrid |
| `aria-colindex`, `aria-rowindex` | cell, columnheader, gridcell, row, rowheader |
| `aria-colspan`, `aria-rowspan` | cell, columnheader, gridcell, rowheader |
| `aria-expanded` | application, button, checkbox, columnheader, combobox\*, gridcell, link, listbox, menuitem, menuitemcheckbox, menuitemradio, row, rowheader, switch, tab, treeitem |
| `aria-level` | heading\*, listitem, row, treeitem |
| `aria-modal` | alertdialog, dialog |
| `aria-multiline`, `aria-placeholder` | searchbox, textbox |
| `aria-multiselectable` | grid, listbox, tablist, tree, treegrid |
| `aria-orientation` | listbox, menu, menubar, radiogroup, scrollbar, separator, slider, tablist, toolbar, tree, treegrid |
| `aria-posinset`, `aria-setsize` | article, listitem, menuitem, menuitemcheckbox, menuitemradio, option, radio, row, tab, treeitem |
| `aria-pressed` | button |
| `aria-readonly` | checkbox, columnheader, combobox, grid, gridcell, listbox, radiogroup, rowheader, searchbox, slider, spinbutton, switch, textbox, treegrid |
| `aria-required` | checkbox, columnheader, combobox, gridcell, listbox, radiogroup, rowheader, searchbox, spinbutton, switch, textbox, tree, treegrid |
| `aria-selected` | columnheader, gridcell, option\*, row, rowheader, tab, treeitem\* |
| `aria-sort` | columnheader, rowheader |
| `aria-valuemax`, `aria-valuemin`, `aria-valuetext` | meter, progressbar, scrollbar, separator, slider, spinbutton |
| `aria-valuenow` | meter\*, progressbar, scrollbar\*, separator\* (if focusable), slider\*, spinbutton |

#### DON'T

```html
<div role="button" tabindex="0" aria-checked="true">Bold</div>
<input type="text" aria-selected="true">
```

#### DO

```html
<div role="button" tabindex="0" aria-pressed="true">Bold</div>
<input type="text" aria-required="true">
```

---

### ARIA-ATTR004 - Never use a state or property the role prohibits

Don't use a state or property the role prohibits, including on elements with an implicit role: a plain `div` or `span` is `generic`, so `<div aria-label="...">` is an error. ([Prohibited States and Properties](https://www.w3.org/TR/wai-aria-1.2/#prohibitedattributes))

| Role | Prohibited |
|---|---|
| `caption`, `code`, `deletion`, `emphasis`, `insertion`, `paragraph`, `presentation` / `none`, `strong`, `subscript`, `superscript` | `aria-label`, `aria-labelledby` |
| `generic` | `aria-label`, `aria-labelledby`, `aria-roledescription` |

#### DON'T

```html
<div class="card" aria-label="Product: Trail running shoes">...</div>
<p aria-label="Warning">Your session expires in 5 minutes.</p>
```

#### DO

```html
<div class="card" role="group" aria-label="Product: Trail running shoes">...</div>
<p><strong>Warning:</strong> your session expires in 5 minutes.</p>
```

---

### ARIA-ATTR005 - Don't use hierarchy or editing attributes on rows and headers of static tables

Don't put `aria-expanded`, `aria-posinset`, `aria-setsize` or `aria-level` on a `row` inside a `table` or `grid`, or `aria-readonly`/`aria-required` on a `columnheader`/`rowheader` inside a `table`. In a native table, use `colspan`/`rowspan`, not `aria-colspan`/`aria-rowspan`. ([row](https://www.w3.org/TR/wai-aria-1.2/#row))

#### DON'T

```html
<div role="table">
  <div role="row" aria-level="1" aria-expanded="true">...</div>
</div>
<table><tr><td aria-colspan="2">Total</td></tr></table>
```

#### DO

```html
<div role="treegrid">
  <div role="row" aria-level="1" aria-expanded="true">...</div>
</div>
<table><tr><td colspan="2">Total</td></tr></table>
```

---

### ARIA-ATTR006 - Don't use aria-checked="mixed" on radio, menuitemradio or switch

Don't use `aria-checked="mixed"` on `radio`, `menuitemradio` or `switch`; `mixed` is for tri-state roles such as `checkbox` and `menuitemcheckbox`. ([aria-checked](https://www.w3.org/TR/wai-aria-1.2/#aria-checked))

Prevailing rule: ARIA-USAGE001 (see C6 in [overlap.md](./overlap.md#conflicts)). The custom `div role="checkbox"` DO example shows valid ARIA, not the recommended control; prefer a native `input type="checkbox"` with its `indeterminate` state.

#### DON'T

```html
<div role="switch" tabindex="0" aria-checked="mixed">Notifications</div>
```

#### DO

```html
<div role="checkbox" tabindex="0" aria-checked="mixed">Select all notifications</div>
```

---

### ARIA-ATTR007 - Write aria-keyshortcuts values in the required key syntax

Write each `aria-keyshortcuts` shortcut as exact modifier names first (`Alt`, `Control`, `Shift`, `Meta`, `AltGraph`), then exactly one non-modifier key, joined by `+`, with shortcuts separated by spaces (e.g. `Shift+Space`, `Alt+Shift+P Control+F`). Write a key produced by a modifier as the physical key (`Shift+5`, not `%`). ([aria-keyshortcuts](https://www.w3.org/TR/wai-aria-1.2/#aria-keyshortcuts))

Automation: Whether a character comes from a modifier depends on the keyboard layouts supported, so it needs manual or AI review.

#### DON'T

```html
<button aria-keyshortcuts="Ctrl+S">Save</button>
<button aria-keyshortcuts="T+Shift+Alt">New tab</button>
```

#### DO

```html
<button aria-keyshortcuts="Control+S">Save</button>
<button aria-keyshortcuts="Alt+Shift+T">New tab</button>
```

---

### ARIA-ATTR008 - Use aria-roledescription sparingly, only on elements with a role, and never empty

Use `aria-roledescription` only to clarify a container such as `group` or `region` or to describe a widget more precisely, only on an element with a valid explicit or implicit role, and never with an empty or whitespace-only value. ([aria-roledescription](https://www.w3.org/TR/wai-aria-1.2/#aria-roledescription))

Automation: Whether the description is appropriate needs manual or AI review.

#### DON'T

```html
<div aria-roledescription="slide" id="slide42">...</div>
<div role="region" aria-roledescription=" " aria-labelledby="slide42heading">...</div>
```

#### DO

```html
<div role="region" aria-roledescription="slide" id="slide42" aria-labelledby="slide42heading">
  <h1 id="slide42heading">Quarterly Report</h1>
</div>
```

---

### ARIA-ATTR009 - Mark only one current item per set and one sorted header per table

Mark only one element per set with `aria-current` (and don't use it in place of `aria-selected`), and set `aria-sort` on only one header per table or grid. ([aria-current](https://www.w3.org/TR/wai-aria-1.2/#aria-current))

Automation: What makes up a set, and whether `aria-current` stands in for `aria-selected`, needs manual or AI review.

#### DON'T

```html
<nav aria-label="Pagination">
  <a href="?page=1" aria-current="page">1</a>
  <a href="?page=2" aria-current="page">2</a>
</nav>
```

#### DO

```html
<nav aria-label="Pagination">
  <a href="?page=1">1</a>
  <a href="?page=2" aria-current="page">2</a>
</nav>
```

---

## Attribute Values

### ARIA-VAL001 - Give each attribute a value of its defined type

Give each state and property a value of its type, and each token attribute one of its listed values. An empty string counts as absent and is allowed on attributes that aren't required. ([Value](https://www.w3.org/TR/wai-aria-1.2/#propcharacteristic_value))

| Type | Valid values | Attributes |
|---|---|---|
| true/false | `true`, `false` | `aria-atomic`, `aria-busy`, `aria-disabled`, `aria-modal`, `aria-multiline`, `aria-multiselectable`, `aria-readonly`, `aria-required` |
| true/false/undefined | `true`, `false`, `undefined` | `aria-expanded`, `aria-grabbed`, `aria-hidden`, `aria-selected` |
| tristate | `true`, `false`, `mixed`, `undefined` | `aria-checked`, `aria-pressed` |
| integer | a number with no fractional part | `aria-colcount`, `aria-colindex`, `aria-colspan`, `aria-level`, `aria-posinset`, `aria-rowcount`, `aria-rowindex`, `aria-rowspan`, `aria-setsize` |
| number | any real number | `aria-valuemax`, `aria-valuemin`, `aria-valuenow` |
| ID reference | one `id` (see ARIA-IDREF001) | `aria-activedescendant`, `aria-details`, `aria-errormessage` |
| ID reference list | space-separated `id`s | `aria-controls`, `aria-describedby`, `aria-flowto`, `aria-labelledby`, `aria-owns` |
| string | unconstrained | `aria-keyshortcuts` (syntax: ARIA-ATTR007), `aria-label`, `aria-placeholder`, `aria-roledescription`, `aria-valuetext` |

| Token attribute | Valid values |
|---|---|
| `aria-autocomplete` | `inline`, `list`, `both`, `none` |
| `aria-current` | `page`, `step`, `location`, `date`, `time`, `true`, `false` |
| `aria-dropeffect` (token list) | `copy`, `execute`, `link`, `move`, `none`, `popup` |
| `aria-haspopup` | `false`, `true`, `menu`, `listbox`, `tree`, `grid`, `dialog` |
| `aria-invalid` | `grammar`, `false`, `spelling`, `true` |
| `aria-live` | `assertive`, `off`, `polite` |
| `aria-orientation` | `horizontal`, `vertical`, `undefined` |
| `aria-relevant` (token list) | `additions`, `removals`, `text`, `all` |
| `aria-sort` | `ascending`, `descending`, `none`, `other` |

#### DON'T

```html
<button aria-expanded="yes">Options</button>
<div aria-live="rude">...</div>
<button aria-haspopup="popup">More</button>
```

#### DO

```html
<button aria-expanded="true">Options</button>
<div aria-live="assertive">...</div>
<button aria-haspopup="menu">More</button>
```

---

### ARIA-VAL002 - Keep integer attributes within their allowed ranges

Keep integer attributes within the constraints below. ([aria-posinset](https://www.w3.org/TR/wai-aria-1.2/#aria-posinset))

| Attribute | Constraint |
|---|---|
| `aria-level` | integer >= 1 |
| `aria-posinset` | integer >= 1, and <= the set size when that size is known |
| `aria-setsize` | the number of items in the set, or `-1` if unknown |
| `aria-colcount`, `aria-rowcount` | the number of columns/rows in the full table, or `-1` if unknown |
| `aria-colindex` | >= 1, greater than any previous `aria-colindex` in the same row, and <= the column count |
| `aria-rowindex` | >= 1, greater than any previous row's `aria-rowindex`, and <= the row count |
| `aria-colspan` | >= 1, and not so large that the cell overlaps the next cell in the row |
| `aria-rowspan` | >= 0 (`0` spans the remaining rows of the row group), and not so large that the cell overlaps the next cell in the column |

Automation: Whether counts and indexes match the full data set, and whether spans overlap, needs manual or AI review.

#### DON'T

```html
<ul role="listbox" aria-label="Available fruit">
  <li role="option" aria-setsize="16" aria-posinset="0">apples</li>
  <li role="option" aria-setsize="16" aria-posinset="17">bananas</li>
</ul>
```

#### DO

```html
<ul role="listbox" aria-label="Available fruit">
  <li role="option" aria-setsize="16" aria-posinset="5">apples</li>
  <li role="option" aria-setsize="16" aria-posinset="6">bananas</li>
</ul>
```

---

### ARIA-VAL003 - Keep range widget values consistent

Keep `aria-valuemax` >= `aria-valuemin` and `aria-valuenow` between them (they default to `0` and `100` on `meter`, `progressbar`, `scrollbar`, `slider` and focusable `separator`). Set `aria-valuenow` whenever `aria-valuetext` is set, unless the value is unknown. ([aria-valuenow](https://www.w3.org/TR/wai-aria-1.2/#aria-valuenow))

#### DON'T

```html
<div role="slider" tabindex="0" aria-label="Volume" aria-valuemin="10" aria-valuemax="0" aria-valuenow="50"></div>
<div role="meter" aria-label="Disk usage" aria-valuenow="130">130%</div>
```

#### DO

```html
<div role="slider" tabindex="0" aria-label="Volume" aria-valuemin="0" aria-valuemax="10" aria-valuenow="5"></div>
<div role="meter" aria-label="Disk usage" aria-valuenow="65">65%</div>
```

---

### ARIA-VAL004 - Describe range values accurately

Make range attributes describe the rendered value:
- Don't set `aria-valuenow` when the value is unknown (e.g. an indeterminate `progressbar`); set it otherwise, including on `spinbutton`.
- Set `aria-valuemin`/`aria-valuemax` when the range is known (on a focusable `separator`, when they aren't `0`/`100`).
- Use `aria-valuetext` only when the value can't be shown meaningfully as a number.

([aria-valuenow](https://www.w3.org/TR/wai-aria-1.2/#aria-valuenow))

Automation: Whether a value is known, bounded or better shown as text needs manual or AI review.

Prevailing rule: HTMLARIA014 in html-aria-rules.md (see C11 in [overlap.md](./overlap.md#conflicts)). This rule applies to explicit range roles only; native `meter`, `progress` and `input` use `min`/`max`, never `aria-valuemin`/`aria-valuemax`.

#### DON'T

```html
<div role="progressbar" aria-label="Connecting" aria-valuenow="0"></div>
<!-- an indeterminate spinner: the progress is unknown, not 0 -->
<div role="slider" tabindex="0" aria-label="Size" aria-valuemin="1" aria-valuemax="3" aria-valuenow="2" aria-valuetext="2"></div>
```

#### DO

```html
<div role="progressbar" aria-label="Connecting"></div>
<div role="slider" tabindex="0" aria-label="Size" aria-valuemin="1" aria-valuemax="3" aria-valuenow="2" aria-valuetext="medium"></div>
```

---

### ARIA-VAL005 - Set position and hierarchy attributes where the spec expects them

Set position and hierarchy attributes when the DOM doesn't convey them:
- Set `aria-level` only when the DOM ancestry doesn't give the right level.
- Use `aria-setsize` together with `aria-posinset`.
- Don't count separators in a menu's `aria-posinset`/`aria-setsize`.
- Put `aria-rowindex` on every row.
- Put `aria-colindex` on every cell, unless the columns are contiguous and no cell spans (then each row is enough).

([aria-posinset](https://www.w3.org/TR/wai-aria-1.2/#aria-posinset))

Automation: Whether the DOM already represents the level needs manual or AI review.

#### DON'T

```html
<div role="grid" aria-rowcount="2000">
  <div role="row" aria-rowindex="1">...</div>
  <div role="row">...</div>  <!-- actually row 100 of the data set -->
</div>
```

#### DO

```html
<div role="grid" aria-rowcount="2000">
  <div role="row" aria-rowindex="1">...</div>
  <div role="row" aria-rowindex="100">...</div>
</div>
```

---

## ID References

### ARIA-IDREF001 - Point every ID reference at an element that exists in the same document

Point every ID reference at an existing element in the same document, and keep IDs unique. `aria-activedescendant`, `aria-details` and `aria-errormessage` take one ID; `aria-controls`, `aria-describedby`, `aria-flowto`, `aria-labelledby` and `aria-owns` take a space-separated list. ([ID Reference Error Processing](https://www.w3.org/TR/wai-aria-1.2/#mapping_additional_relations_error_processing))

Overlaps: ARIA1, ARIA16, IDS001 in wcag2.2-rules.md

#### DON'T

```html
<input type="text" aria-labelledby="startTime-label">
<!-- no element has id="startTime-label" -->
<button aria-controls="panel-1 panel-2">Expand all</button>
<!-- only panel-1 exists -->
```

#### DO

```html
<span id="startTime-label">Start time</span>
<input type="text" aria-labelledby="startTime-label">
<button aria-controls="panel-1 panel-2">Expand all</button>
<div id="panel-1">...</div>
<div id="panel-2">...</div>
```

---

### ARIA-IDREF002 - Give each element at most one aria-owns owner

Don't list an element's ID in more than one element's `aria-owns`, and don't create circular ownership. ([aria-owns](https://www.w3.org/TR/wai-aria-1.2/#aria-owns))

#### DON'T

```html
<div role="tree" aria-owns="node-7">...</div>
<div role="group" aria-owns="node-7">...</div>
<div role="treeitem" id="node-7" aria-selected="false">Reports</div>
```

#### DO

```html
<div role="tree" aria-label="Files">
  <div role="group" aria-owns="node-7"></div>
</div>
<div role="treeitem" id="node-7" aria-selected="false">Reports</div>
```

---

### ARIA-IDREF003 - Point aria-activedescendant at an owned element

Point `aria-activedescendant` at an element its element owns (a DOM descendant or through `aria-owns`), or, on a `combobox`, `textbox` or `searchbox`, at an element owned by the element its `aria-controls` references. ([aria-activedescendant](https://www.w3.org/TR/wai-aria-1.2/#aria-activedescendant))

Automation: The requirement applies while the element has DOM focus, which a static render can't observe.

#### DON'T

```html
<ul role="listbox" tabindex="0" aria-label="Tags" aria-activedescendant="opt-zoom">
  <li role="option" aria-selected="false">Zebra</li>
</ul>
<li role="option" id="opt-zoom" aria-selected="false">Zoom</li>
```

#### DO

```html
<label for="tag_combo">Tag</label>
<input type="text" id="tag_combo" role="combobox" aria-autocomplete="list"
  aria-haspopup="listbox" aria-expanded="true"
  aria-controls="popup_listbox" aria-activedescendant="selected_option">
<ul role="listbox" id="popup_listbox">
  <li role="option" aria-selected="false">Zebra</li>
  <li role="option" id="selected_option" aria-selected="true">Zoom</li>
</ul>
```

---

### ARIA-IDREF004 - Pair aria-errormessage with aria-invalid, and hide the message while the value is valid

Use `aria-errormessage` only together with `aria-invalid`, and show the referenced message only while `aria-invalid="true"` (otherwise hide it or remove the reference). ([aria-errormessage](https://www.w3.org/TR/wai-aria-1.2/#aria-errormessage))

Automation: Showing and hiding the message as validity changes needs JS execution and interaction.

Prevailing rule: ARIA-LIVE001 (see C12 in [overlap.md](./overlap.md#conflicts)). The spec example's `aria-live="assertive"` is kept as written but is not a default to copy; a validation message is usually polite unless the interruption is imperative.

#### DON'T

```html
<input id="startTime" type="text" aria-errormessage="msgID" value="">
<span id="msgID">Invalid time: the time must be between 9:00 AM and 5:00 PM</span>
```

#### DO

```html
<input id="startTime" type="text" aria-errormessage="msgID" value="" aria-invalid="false">
<span id="msgID" aria-live="assertive"><span style="visibility:hidden">Invalid time: the time must be between 9:00 AM and 5:00 PM</span></span>
```

---

### ARIA-IDREF005 - Reference every tooltip from aria-describedby

Reference every `role="tooltip"` element from an `aria-describedby`, at the latest when it is displayed. ([tooltip](https://www.w3.org/TR/wai-aria-1.2/#tooltip))

Automation: Whether the reference exists by the time the tooltip is displayed needs JS execution and interaction.

Overlaps: ARIA1 in wcag2.2-rules.md

#### DON'T

```html
<button>Save</button>
<div role="tooltip" id="save-tip">Saves the draft without publishing</div>
```

#### DO

```html
<button aria-describedby="save-tip">Save</button>
<div role="tooltip" id="save-tip">Saves the draft without publishing</div>
```

---

## Required Context & Owned Elements

### ARIA-STRUCT001 - Place roles with a required context inside that context

Place each role below inside (or owned by) an element with its required context role, explicit or implicit. An `option`'s `group` must be inside a `listbox`, and a `menuitemradio`'s `group` inside a `menu` or `menubar`. ([Required Context Role](https://www.w3.org/TR/wai-aria-1.2/#scope))

| Role | Required context |
|---|---|
| `caption` | `figure`, `grid`, `table`, `treegrid` |
| `cell`, `columnheader`, `gridcell`, `rowheader` | `row` |
| `listitem` | `list`, `directory` |
| `menuitem`, `menuitemcheckbox`, `menuitemradio` | `menu`, `menubar`, or a `group` inside one |
| `option` | `listbox`, or a `group` inside one |
| `row` | `grid`, `rowgroup`, `table`, `treegrid` |
| `rowgroup` | `grid`, `table`, `treegrid` |
| `tab` | `tablist` |
| `treeitem` | `tree`, `group` |

#### DON'T

```html
<div role="tab" aria-selected="true">General</div>
<ul>
  <li role="option" aria-selected="false">Zebra</li>
</ul>
```

#### DO

```html
<div role="tablist" aria-label="Settings">
  <div role="tab" aria-selected="true">General</div>
</div>
<ul role="listbox" aria-label="Animals">
  <li role="option" aria-selected="false">Zebra</li>
</ul>
```

---

### ARIA-STRUCT002 - Give container roles their required owned elements

Give each container role below at least one owned element with a listed role (subclass roles don't count), unless a containing element has `aria-busy="true"` while it loads. (`A → B` means a B inside an A.) ([Required Owned Elements](https://www.w3.org/TR/wai-aria-1.2/#mustContain))

| Role | Required owned elements (at least one) |
|---|---|
| `feed` | `article` |
| `grid`, `table`, `treegrid` | `row`, or `rowgroup → row` |
| `list` | `listitem` |
| `listbox` | `option`, or `group → option` |
| `menu`, `menubar` | `menuitem`, `menuitemcheckbox`, `menuitemradio`, or `group →` one of these |
| `radiogroup` | `radio` |
| `row` | `cell`, `columnheader`, `gridcell`, `rowheader` |
| `rowgroup` | `row` |
| `tablist` | `tab` |
| `tree` | `treeitem`, or `group → treeitem` |

Automation: Whether a container is missing owned elements only while loading needs JS execution; see ARIA-STATE005.

#### DON'T

```html
<div role="tablist" aria-label="Settings">
  <button>General</button>
  <button>Privacy</button>
</div>
```

#### DO

```html
<div role="tablist" aria-label="Settings">
  <button role="tab" aria-selected="true">General</button>
  <button role="tab" aria-selected="false">Privacy</button>
</div>
```

---

### ARIA-STRUCT003 - Limit what listbox groups and spinbuttons contain

Limit a `group` inside a `listbox` to `option` children, and a `spinbutton`'s children to a `textbox` and/or two buttons. ([group](https://www.w3.org/TR/wai-aria-1.2/#group))

#### DON'T

```html
<ul role="listbox" aria-label="Fruit">
  <li role="group" aria-label="Citrus">
    <span role="heading" aria-level="3">Citrus</span>
    <span role="option" aria-selected="false">Lemon</span>
  </li>
</ul>
```

#### DO

```html
<ul role="listbox" aria-label="Fruit">
  <li role="group" aria-label="Citrus">
    <span role="option" aria-selected="false">Lemon</span>
    <span role="option" aria-selected="false">Lime</span>
  </li>
</ul>
```

---

### ARIA-STRUCT004 - Make a caption the first child of its table, and name the table from it

Make a `caption` the first child of its `table`, `grid` or `treegrid`, or the first or last child of its `figure`, and reference it from the parent with `aria-labelledby`. ([caption](https://www.w3.org/TR/wai-aria-1.2/#caption))

#### DON'T

```html
<div role="table">
  <div role="rowgroup">...</div>
  <div role="caption">Contest Entrants</div>
</div>
```

#### DO

```html
<div role="table" aria-labelledby="name" aria-describedby="desc">
  <div role="caption">
    <div id="name">Contest Entrants</div>
    <div id="desc">This table shows the total number of entrants (500) the contest accepted over the past four weeks.</div>
  </div>
  <div role="rowgroup">...</div>
</div>
```

---

## Accessible Names

### ARIA-NAME001 - Give every role that requires a name an accessible name

Give every role below a non-empty accessible name, from its contents where allowed, or from `aria-labelledby`, `aria-label` or a host-language mechanism (`label`, `alt`, `title`). Also name each `toolbar`, and each focusable `separator`, when there is more than one. ([Accessible Name Calculation](https://www.w3.org/TR/wai-aria-1.2/#namecalculation))

| Name source | Roles that require a name |
|---|---|
| Contents or author | `button`, `checkbox`, `columnheader`, `heading`, `link`, `menuitem`, `menuitemcheckbox`, `menuitemradio`, `option`, `radio`, `rowheader`, `switch`, `tooltip`, `treeitem` |
| Author only | `alertdialog`, `application`, `combobox`, `dialog`, `form`, `grid`, `img`, `listbox`, `marquee`, `meter`, `progressbar`, `radiogroup`, `region`, `searchbox`, `slider`, `spinbutton`, `table`, `tabpanel`, `textbox`, `tree`, `treegrid` |

Automation: Whether the name describes the purpose needs manual or AI review.

Overlaps: F68, ARIA14, ARIA16 in wcag2.2-rules.md

Prevailing rules: H51, H63, F91 in wcag2.2-rules.md and a25f45 in act-rules.md (see C4 in [overlap.md](./overlap.md#conflicts)). The name requirement applies to explicit roles only; a native `table` without a name is not a failure (a name is a suggestion for AI review).

#### DON'T

```html
<div role="dialog" aria-modal="true">
  <h2>Delete file?</h2>
  ...
</div>
<div role="img">
  <img src="chart-part1.png" alt=""><img src="chart-part2.png" alt="">
</div>
```

#### DO

```html
<div role="dialog" aria-modal="true" aria-labelledby="dlg-title">
  <h2 id="dlg-title">Delete file?</h2>
  ...
</div>
<div role="img" aria-label="Sales by quarter, 2026">
  <img src="chart-part1.png" alt=""><img src="chart-part2.png" alt="">
</div>
```

---

## Presentational Roles

### ARIA-PRES001 - Don't put role="none"/"presentation" on focusable elements or elements with global ARIA attributes

Don't put `role="none"`/`"presentation"` on a focusable element or on one with global ARIA attributes; user agents ignore the role there. ([Presentational Roles Conflict Resolution](https://www.w3.org/TR/wai-aria-1.2/#conflict_resolution_presentation_none))

#### DON'T

```html
<a href="/checkout" role="presentation">Checkout</a>
<h1 role="presentation" aria-describedby="comment-1">Sample Content</h1>
```

#### DO

```html
<a href="/checkout">Checkout</a>
<ul role="tree" aria-label="Files">
  <li role="presentation">
    <a role="treeitem" aria-expanded="true" aria-selected="false">An expanded tree node</a>
  </li>
</ul>
```

---

### ARIA-PRES002 - Don't give a presentational image meaningful alt text

Give an image with `role="presentation"`/`"none"` empty alt text (`alt=""`); if it carries meaning, remove the role or name a container with role `img`. ([presentation](https://www.w3.org/TR/wai-aria-1.2/#presentation))

Automation: Whether the alt text is meaningful needs manual or AI review.

Overlaps: H67 in wcag2.2-rules.md

#### DON'T

```html
<img src="q3-sales.png" role="presentation" alt="Sales grew 20% in Q3">
```

#### DO

```html
<div role="img" aria-labelledby="caption">
  <img src="example.png" role="presentation" alt="">
  <p id="caption">A visible text caption labeling the image.</p>
</div>
```

---

### ARIA-PRES003 - Don't put structural or interactive content inside roles with presentational children

Don't put semantic roles (headings, lists, tables, links) or focusable elements inside roles whose children are presentational: `button`, `checkbox`, `img`, `menuitemcheckbox`, `menuitemradio`, `meter`, `option`, `progressbar`, `radio`, `scrollbar`, `separator`, `slider`, `switch`, `tab`. ([Presentational Children](https://www.w3.org/TR/wai-aria-1.2/#childrenArePresentational))

#### DON'T

```html
<div role="button" tabindex="0">
  <h3>Pro plan</h3>
  <ul><li>100 GB storage</li><li>Priority support</li></ul>
</div>
<div role="tab" aria-selected="false"><a href="/details">Details</a></div>
```

#### DO

```html
<div role="button" tabindex="0">Choose the Pro plan</div>
<div role="tab" aria-selected="false">Details</div>
```

---

## Hidden Content

### ARIA-HIDDEN001 - Don't hide focusable or functional content with aria-hidden

Don't put focusable or functional content inside `aria-hidden="true"`, and don't use `aria-hidden="false"` inside a hidden subtree. Hide visible content only when it is redundant or extraneous. ([aria-hidden](https://www.w3.org/TR/wai-aria-1.2/#aria-hidden))

Automation: Whether the hidden content's meaning is exposed some other way needs manual or AI review.

#### DON'T

```html
<div aria-hidden="true">
  <a href="/checkout">Go to checkout</a>
</div>
```

#### DO

```html
<a href="/checkout">
  <svg aria-hidden="true"><!-- cart icon --></svg>
  Go to checkout
</a>
```

---

## Widgets

### ARIA-WIDGET001 - Build comboboxes with the ARIA 1.2 pattern

Put `role="combobox"` on the input itself, reference its popup with `aria-controls` (not `aria-owns`), give the popup role `listbox`, `tree`, `grid` or `dialog`, set `aria-haspopup` for any popup other than `listbox`, and always set `aria-expanded`. ([combobox](https://www.w3.org/TR/wai-aria-1.2/#combobox))

Automation: Whether `aria-expanded` matches the popup's visibility needs JS execution and interaction; see ARIA-WIDGET004.

#### DON'T

```html
<div role="combobox" aria-expanded="false" aria-owns="country-list">
  <input type="text" aria-label="Country">
</div>
<ul role="listbox" id="country-list">...</ul>
```

#### DO

```html
<label for="tag_combo">Tag</label>
<input type="text" id="tag_combo" role="combobox" aria-autocomplete="list"
  aria-haspopup="listbox" aria-expanded="true"
  aria-controls="popup_listbox" aria-activedescendant="selected_option">
<ul role="listbox" id="popup_listbox">
  <li role="option" aria-selected="false">Zebra</li>
  <li role="option" id="selected_option" aria-selected="true">Zoom</li>
</ul>
```

---

### ARIA-WIDGET002 - Match aria-haspopup to the role of the popup

Give a popup container role `menu`, `listbox`, `tree`, `grid` or `dialog`, and make the trigger's `aria-haspopup` match it (`true` means `menu`). ([aria-haspopup](https://www.w3.org/TR/wai-aria-1.2/#aria-haspopup))

Automation: When the trigger has no `aria-controls`, finding the popup needs JS execution and interaction.

#### DON'T

```html
<button aria-haspopup="true" aria-controls="share-dialog">Share</button>
<div role="dialog" id="share-dialog" aria-label="Share">...</div>
```

#### DO

```html
<button aria-haspopup="dialog" aria-controls="share-dialog">Share</button>
<div role="dialog" id="share-dialog" aria-label="Share">...</div>
```

---

### ARIA-WIDGET003 - Connect list-style autocomplete to its suggestions popup

With `aria-autocomplete="list"` or `"both"`, set `aria-controls` to the suggestions container and `aria-haspopup` to match its role (implicit `listbox` on `combobox`, explicit on `textbox` and `searchbox`). ([aria-autocomplete](https://www.w3.org/TR/wai-aria-1.2/#aria-autocomplete))

#### DON'T

```html
<input type="search" aria-label="Search products" aria-autocomplete="list">
<ul role="listbox" id="suggestions">...</ul>
```

#### DO

```html
<input type="search" aria-label="Search products" aria-autocomplete="list"
  aria-controls="suggestions" aria-haspopup="listbox">
<ul role="listbox" id="suggestions">...</ul>
```

---

### ARIA-WIDGET004 - Make combobox behavior match its markup

Make a combobox's markup match its behavior:
- Set `aria-autocomplete` to the autocompletion it actually provides.
- Set `aria-expanded` to `true` while the popup is shown and `false` otherwise.
- Make a separate popup-opening icon a `button`, outside the combobox and out of the Tab order.
- Provide keys to move between the combobox and the popup, or manage `aria-activedescendant`.

([combobox](https://www.w3.org/TR/wai-aria-1.2/#combobox))

Automation: Whether `aria-expanded` and `aria-autocomplete` match the real behavior, and the keyboard support, need JS execution and interaction.

#### DON'T

```html
<input type="text" role="combobox" aria-label="Country" aria-expanded="false" aria-controls="countries">
<button aria-label="Show countries"><svg aria-hidden="true"><!-- chevron --></svg></button>
<ul role="listbox" id="countries">...</ul>
<!-- script filters the list as the user types and toggles the popup without updating aria-expanded -->
```

#### DO

```html
<input type="text" role="combobox" aria-label="Country" aria-autocomplete="list"
  aria-expanded="false" aria-controls="countries">
<button tabindex="-1" aria-label="Show countries"><svg aria-hidden="true"><!-- chevron --></svg></button>
<ul role="listbox" id="countries">...</ul>
<!-- script sets aria-expanded="true"/"false" as the listbox opens and closes; Down Arrow moves into it -->
```

---

### ARIA-WIDGET005 - Make autocomplete suggestions behave the way aria-autocomplete says

Make suggestions behave as `aria-autocomplete` says:
- Omit it (or set `none`) when suggestions don't depend on what the user typed.
- With `inline` or `both`, show the completion as selected text.
- With `list` or `both` and automatic selection, keep suggestions in a role that supports `aria-activedescendant`, update `aria-activedescendant`, and keep DOM focus on the input.
- Don't change the input's value to signal a suggestion; with `list` or `both`, use `aria-expanded` to show whether suggestions are displayed.

([aria-autocomplete](https://www.w3.org/TR/wai-aria-1.2/#aria-autocomplete))

Automation: Suggestion behavior (selection, focus, value changes) needs JS execution and interaction.

#### DON'T

```html
<input type="search" role="combobox" aria-label="Search" aria-autocomplete="list"
  aria-expanded="false" aria-controls="recent">
<ul role="listbox" id="recent" aria-label="Recent searches">...</ul>
<!-- always lists the 5 most recent searches, whatever the user types -->
```

#### DO

```html
<input type="search" role="combobox" aria-label="Search" aria-autocomplete="none"
  aria-expanded="false" aria-controls="recent">
<ul role="listbox" id="recent" aria-label="Recent searches">...</ul>
```

---

### ARIA-WIDGET006 - Make popup triggers keyboard-operable and visibly marked

Make popup triggers usable from the keyboard and visibly marked:
- Make the trigger focusable and openable from the keyboard, and manage focus inside the popup.
- Use `aria-haspopup` only when a visual indicator (e.g. a chevron) shows that a popup opens.
- Open a submenu when a `menuitem` with `aria-haspopup` is activated.

([aria-haspopup](https://www.w3.org/TR/wai-aria-1.2/#aria-haspopup))

Automation: The visual indicator needs manual or AI review, and keyboard opening and focus management need JS execution and interaction.

#### DON'T

```html
<span aria-haspopup="menu" onclick="openMenu()">Account</span>
```

#### DO

```html
<button aria-haspopup="menu" aria-expanded="false" aria-controls="account-menu">
  Account <svg aria-hidden="true"><!-- chevron --></svg>
</button>
```

---

### ARIA-WIDGET007 - Keep feeds readable while articles load

In a `feed`:
- Set `aria-busy="true"` while adding or removing articles, then back to `false`.
- Don't insert or remove articles in the middle.
- Make each article focusable, and scroll it into view when it or a descendant gets focus.
- Load more articles before focus reaches either end, or provide a button to load more.
- Provide keyboard commands to move focus between articles.

([feed](https://www.w3.org/TR/wai-aria-1.2/#feed))

Automation: Busy toggling, insertion position, scrolling, preloading and keyboard commands need JS execution and interaction.

#### DON'T

```html
<div role="feed" aria-label="News">
  <article aria-labelledby="a1-title"><h2 id="a1-title">...</h2>...</article>
</div>
```

#### DO

```html
<div role="feed" aria-label="News" aria-busy="false">
  <article tabindex="0" aria-labelledby="a1-title" aria-describedby="a1-summary">
    <h2 id="a1-title">...</h2><p id="a1-summary">...</p>
  </article>
</div>
```

---

## Keyboard & Focus

### ARIA-FOCUS001 - Make interactive elements focusable and composite widgets a single Tab stop

Make every interactive element focusable, and each composite widget a single Tab stop with its own navigation (usually arrow keys) to its parts. Manage focus in `grid`, `listbox`, `menu`, `menubar`, `radiogroup`, `tablist`, `tree`, `treegrid` and `spinbutton` (by moving DOM focus or with `aria-activedescendant`), and keep read-only elements navigable. ([Managing Focus](https://www.w3.org/TR/wai-aria-1.2/#managingfocus_authors))

Automation: Arrow-key navigation and focus tracking need JS execution and interaction.

#### DON'T

```html
<div role="tablist" aria-label="Settings">
  <div role="tab" aria-selected="true">General</div>
  <div role="tab" aria-selected="false">Privacy</div>
</div>
```

#### DO

```html
<div role="tablist" aria-label="Settings">
  <div role="tab" tabindex="0" aria-selected="true">General</div>
  <div role="tab" tabindex="-1" aria-selected="false">Privacy</div>
</div>
<!-- script moves tabindex="0" and focus between tabs on Left/Right Arrow -->
```

---

### ARIA-FOCUS002 - Keep focus on a logical element when content changes

When removing the focused element, move focus to a logical element, and don't scroll the focused element off screen unless the user scrolled. ([Managing Focus](https://www.w3.org/TR/wai-aria-1.2/#managingfocus_authors))

Automation: Nothing can be checked statically: it needs JS execution and interaction, or manual/AI review of the scripts.

#### DON'T

```html
<script>
  function deleteRow(button) {
    button.closest('tr').remove(); // focus is lost with the removed button
  }
</script>
```

#### DO

```html
<script>
  function deleteRow(button) {
    const row = button.closest('tr');
    const next = row.nextElementSibling || row.previousElementSibling;
    row.remove();
    next?.querySelector('button')?.focus();
  }
</script>
```

---

### ARIA-FOCUS003 - Keep the aria-activedescendant target visible and styled as focused

With `aria-activedescendant`, style the active descendant as focused without relying on `:focus`, keep it scrolled into view, and update the value as the user moves (see ARIA-IDREF003). ([aria-activedescendant](https://www.w3.org/TR/wai-aria-1.2/#aria-activedescendant))

Automation: Updating the value, scrolling and the visible indicator need JS execution and interaction.

#### DON'T

```html
<style>[role="option"]:focus { outline: 2px solid; }</style>
<ul role="listbox" tabindex="0" aria-label="Fruit" aria-activedescendant="opt-2">
  <li role="option" id="opt-1" aria-selected="false">Apple</li>
  <li role="option" id="opt-2" aria-selected="false">Banana</li>
</ul>
```

#### DO

```html
<style>[role="option"].active { outline: 2px solid; }</style>
<ul role="listbox" tabindex="0" aria-label="Fruit" aria-activedescendant="opt-2">
  <li role="option" id="opt-1" aria-selected="false">Apple</li>
  <li role="option" id="opt-2" class="active" aria-selected="false">Banana</li>
</ul>
```

---

### ARIA-FOCUS004 - Put grid focus on cells, with an interaction mode for complex cells

When navigating a `grid` by keyboard, focus a `gridcell`, `rowheader` or `columnheader`, unless the cell holds a single widget that doesn't use arrow keys. Provide an interaction or edit mode for cells with an arrow-key widget, several interactive elements or editable content. ([grid](https://www.w3.org/TR/wai-aria-1.2/#grid))

Automation: Whether the grid offers an interaction mode, and where focus lands, needs JS execution and interaction.

#### DON'T

```html
<div role="gridcell" tabindex="-1">
  <button>Edit</button><button>Delete</button>
</div>
<!-- arrow keys move between cells; there is no way to reach the second button -->
```

#### DO

```html
<div role="gridcell" tabindex="-1">
  <button tabindex="-1">Edit</button><button tabindex="-1">Delete</button>
</div>
<!-- Enter switches to interaction mode (Tab moves between the buttons), Escape returns to grid navigation -->
```

---

### ARIA-FOCUS005 - Give spinbuttons their expected keyboard behavior

When a `spinbutton` gets focus, focus its textbox if there is one. Make Up/Down Arrow increment and decrement, and keep the increment and decrement buttons out of the Tab order. ([spinbutton](https://www.w3.org/TR/wai-aria-1.2/#spinbutton))

Automation: Focus placement and arrow-key behavior need JS execution and interaction.

#### DON'T

```html
<button onclick="step(-1)">-</button>
<input type="text" role="spinbutton" aria-label="Quantity" aria-valuenow="1" aria-valuemin="1">
<button onclick="step(1)">+</button>
```

#### DO

```html
<button tabindex="-1" aria-label="Decrease quantity" onclick="step(-1)">-</button>
<input type="text" role="spinbutton" aria-label="Quantity" aria-valuenow="1" aria-valuemin="1">
<button tabindex="-1" aria-label="Increase quantity" onclick="step(1)">+</button>
<!-- script handles Up/Down Arrow on the input -->
```

---

### ARIA-FOCUS006 - Move focus into dialogs and keep modal interaction inside them

Manage focus in dialogs:
- Give every dialog at least one focusable descendant.
- Move focus into a modal dialog or `alertdialog` when it opens, and manage focus inside it.
- Make `alertdialog` modal.
- With `aria-modal="true"`, keep every control the user needs inside the dialog, and make the rest of the page inert.

([dialog](https://www.w3.org/TR/wai-aria-1.2/#dialog))

Automation: Initial focus, focus containment and inert content need JS execution and interaction.

#### DON'T

```html
<button onclick="closeDialog()">Close</button>
<div role="dialog" aria-modal="true" aria-labelledby="dlg-title">
  <h2 id="dlg-title">Photo details</h2>
  <p>Taken on 12 May 2026.</p>
</div>
```

#### DO

```html
<div role="dialog" aria-modal="true" aria-labelledby="dlg-title">
  <h2 id="dlg-title">Photo details</h2>
  <p>Taken on 12 May 2026.</p>
  <button onclick="closeDialog()">Close</button>
</div>
<!-- script focuses the Close button on open and makes the rest of the page inert -->
```

---

### ARIA-FOCUS007 - Implement, expose and scope every aria-keyshortcuts shortcut

Implement every `aria-keyshortcuts` shortcut in script, make it discoverable (e.g. in a tooltip), make it unavailable when its element is disabled, and don't override operating system, browser or assistive technology keys. ([aria-keyshortcuts](https://www.w3.org/TR/wai-aria-1.2/#aria-keyshortcuts))

Automation: Whether a handler exists, follows the disabled state and avoids reserved keys needs JS execution and interaction, or manual/AI review.

#### DON'T

```html
<button aria-keyshortcuts="Control+P">Publish</button>
<!-- no keydown handler; Control+P is also the browser's Print shortcut -->
```

#### DO

```html
<button aria-keyshortcuts="Alt+Shift+P" title="Publish (Alt+Shift+P)">Publish</button>
<script>
  document.addEventListener('keydown', e => {
    if (e.altKey && e.shiftKey && e.code === 'KeyP' && !publishButton.disabled) publish();
  });
</script>
```

---

## Dynamic States

### ARIA-STATE001 - Update states and values whenever the UI changes

Update states and values whenever the UI changes:
- `combobox`: set `aria-expanded` as the popup opens and closes.
- Focusable `separator`: update `aria-valuenow` when it moves.
- `progressbar`: update `aria-valuenow` with the visual indicator.
- `timer`: update the text at fixed intervals, except when paused or finished.
- Apply state changes requested by assistive technologies.

([WAI-ARIA States and Properties](https://www.w3.org/TR/wai-aria-1.2/#introstates))

Automation: Nothing can be checked statically: it needs JS execution and interaction, or manual/AI review of the event handlers.

#### DON'T

```html
<div role="separator" tabindex="0" aria-label="Resize sidebar" aria-valuenow="30"></div>
<script>
  function resize(percent) {
    sidebar.style.width = percent + '%'; // aria-valuenow stays at 30
  }
</script>
```

#### DO

```html
<div role="separator" tabindex="0" aria-label="Resize sidebar" aria-valuenow="30"></div>
<script>
  function resize(percent) {
    sidebar.style.width = percent + '%';
    splitter.setAttribute('aria-valuenow', percent);
  }
</script>
```

---

### ARIA-STATE002 - Keep checked and selected states consistent within a set

Keep selection states consistent within a set:
- Check only one `radio` (or `menuitemradio`) per group.
- In a `tablist`, set `aria-selected="true"` on the selected tab and `false` on the others, show the selection visually, and hide unselected tabs' panels (in a multi-selectable tablist, use `aria-expanded` instead).
- In an `aria-multiselectable` container, set `aria-selected` on every selectable item and on nothing else; set `aria-multiselectable="true"` on a grid that allows multiple selection.

([aria-selected](https://www.w3.org/TR/wai-aria-1.2/#aria-selected))

Automation: Consistency after interaction and the visual indication need JS execution and interaction.

Prevailing rule: ARIA-USAGE001 (see C6 in [overlap.md](./overlap.md#conflicts)). The custom `div role="radio"` examples show valid ARIA, not the recommended control; prefer native `input type="radio"` elements.

#### DON'T

```html
<div role="radiogroup" aria-label="Shipping">
  <div role="radio" tabindex="0" aria-checked="true">Standard</div>
  <div role="radio" tabindex="-1" aria-checked="true">Express</div>
</div>
```

#### DO

```html
<div role="radiogroup" aria-label="Shipping">
  <div role="radio" tabindex="0" aria-checked="true">Standard</div>
  <div role="radio" tabindex="-1" aria-checked="false">Express</div>
</div>
```

---

### ARIA-STATE003 - Show disabled state visually, and use aria-disabled only where it can work

Make disabled elements look disabled, and don't use `aria-disabled` on elements the host language can't disable (e.g. links), since it disables nothing by itself. ([aria-disabled](https://www.w3.org/TR/wai-aria-1.2/#aria-disabled))

Automation: Blocking activation needs JS execution and interaction.

Prevailing rule: HTMLARIA011 in html-aria-rules.md (see C7 in [overlap.md](./overlap.md#conflicts)). Only `aria-disabled` on an `a` with `href` is flagged; `<a role="link" aria-disabled="true">` without `href` is the documented pattern for a disabled link.

#### DON'T

```html
<a href="/export" aria-disabled="true">Export</a>
```

#### DO

```html
<button type="button" disabled>Export</button>
```

---

### ARIA-STATE004 - Set aria-invalid only for detected errors, and suggest corrections

Set `aria-invalid="true"` when a value fails validation and suggest corrections when known, but don't set it on empty required fields before the user submits. ([aria-invalid](https://www.w3.org/TR/wai-aria-1.2/#aria-invalid))

Automation: Validation timing and the quality of suggestions need JS execution and interaction, or manual/AI review.

#### DON'T

```html
<!-- on page load, before the user has typed anything -->
<input id="email" type="email" required aria-invalid="true" aria-errormessage="email-err" value="">
<p id="email-err">Enter an email address</p>
```

#### DO

```html
<!-- after a failed submit -->
<input id="email" type="email" required aria-invalid="true" aria-errormessage="email-err" value="jane.example.com">
<p id="email-err">Enter an email address with an @, e.g. jane@example.com</p>
```

---

### ARIA-STATE005 - Mark widgets and regions busy while they are being updated

Set `aria-busy="true"` on a widget while an update leaves it missing required owned elements, and on a region while it loads (pointing to its `progressbar` with `aria-describedby`). ([aria-busy](https://www.w3.org/TR/wai-aria-1.2/#aria-busy))

Automation: Nothing can be checked statically: it needs JS execution to observe updates, or manual/AI review of the scripts.

#### DON'T

```html
<section aria-labelledby="results-h">
  <h2 id="results-h">Results</h2>
  <div role="progressbar" aria-label="Loading results" aria-valuenow="40"></div>
</section>
```

#### DO

```html
<section aria-labelledby="results-h" aria-busy="true" aria-describedby="results-progress">
  <h2 id="results-h">Results</h2>
  <div role="progressbar" id="results-progress" aria-label="Loading results" aria-valuenow="40"></div>
</section>
<!-- script sets aria-busy="false" once the results are in place -->
```

---

## Live Regions

### ARIA-LIVE001 - Reserve assertive announcements and alerts for urgent information

Use `aria-live="assertive"` and `role="alert"` only when the interruption is imperative, don't make users close an alert (use `alertdialog` when focus should move), and use `aria-relevant` `removals`/`all` sparingly. ([aria-live](https://www.w3.org/TR/wai-aria-1.2/#aria-live))

Automation: Whether the interruption is imperative needs manual or AI review.

#### DON'T

```html
<div role="alert">
  Your preferences were saved.
  <button onclick="dismiss()">OK</button>
</div>
```

#### DO

```html
<div role="status">Your preferences were saved.</div>
```

---

### ARIA-LIVE002 - Keep status messages out of the focus flow and linked to what controls them

Don't move focus to a `role="status"` element when it changes, and reference it with `aria-controls` from whatever controls it. ([status](https://www.w3.org/TR/wai-aria-1.2/#status))

Automation: Whether script moves focus to the status needs JS execution and interaction.

#### DON'T

```html
<button onclick="save()">Save</button>
<div role="status" id="save-status" tabindex="-1"></div>
<script>
  function save() { saveStatus.textContent = 'Saved'; saveStatus.focus(); }
</script>
```

#### DO

```html
<button aria-controls="save-status" onclick="save()">Save</button>
<div role="status" id="save-status"></div>
<script>
  function save() { saveStatus.textContent = 'Saved'; }
</script>
```

---

## Landmarks

### ARIA-LMK001 - Mark at most one banner, main and contentinfo per document

Mark at most one `banner`, one `main` and one `contentinfo` per document (explicit or implicit role); a nested `document` or `application` may have its own. ([main](https://www.w3.org/TR/wai-aria-1.2/#main))

#### DON'T

```html
<div role="main">...article...</div>
<div role="main">...related articles...</div>
```

#### DO

```html
<div role="main">...article...</div>
<div role="complementary" aria-label="Related articles">...</div>
```

---

## Semantic Choices

### ARIA-USAGE001 - Prefer native host-language elements over repurposed ones

Use a native host-language element (e.g. `<input type="checkbox">`, `<table>`) instead of repurposing another element with a role, unless there is a compelling reason. ([Conflicts with Host Language Semantics](https://www.w3.org/TR/wai-aria-1.2/#host_general_conflict))

Automation: Whether there is a compelling reason needs manual or AI review.

#### DON'T

```html
<div role="checkbox" tabindex="0" aria-checked="false">Remember me</div>
```

#### DO

```html
<label><input type="checkbox"> Remember me</label>
```

---

### ARIA-USAGE002 - Pick the role that matches the content's purpose

Pick the role that matches the content's purpose. ([table](https://www.w3.org/TR/wai-aria-1.2/#table))

| Choice | Requirement |
|---|---|
| `table` vs `grid`/`treegrid` | Use `grid`/`treegrid` when it keeps a selection, has its own two-dimensional navigation, or lets users rearrange or edit its contents. |
| `meter` vs `progressbar` | Don't use `meter` to show progress. |
| `form` vs `search` | Use `search` for a form that submits search criteria. |
| `region` | Use only for content no other landmark role describes. |
| `group` vs `region` | Use `region` (or another landmark) for a section important enough for the page's table of contents. |
| `link` vs `button` | Use `button` when activating it triggers an action without changing focus or page location. |
| `emphasis`, `strong`, `subscript`, `superscript` | Use only when removing them would change the meaning, not for typographic presentation. |
| `term` | Don't use on interactive elements such as links. |
| `time` | Limit the text to a valid date or time string (e.g. `2019-11-18`, `09:54:39`, `4h 18m 3s`). |

Automation: Every choice except `term` and `time` depends on purpose and behavior, so it needs manual or AI review.

#### DON'T

```html
<div role="meter" aria-label="Upload progress" aria-valuenow="40">40%</div>
<a href="#" onclick="openFilters()">Filters</a>
```

#### DO

```html
<div role="progressbar" aria-label="Upload progress" aria-valuenow="40">40%</div>
<button type="button" onclick="openFilters()">Filters</button>
```

---

### ARIA-USAGE003 - Name and describe elements with the right mechanism

Name and describe elements with the right mechanism:
- Use `aria-labelledby` when the label text is in the DOM, and `aria-label` only when no visible label is possible.
- For `form` and `region`, reference a visible label (ideally a heading) with `aria-labelledby`.
- Don't use `aria-placeholder` instead of a label, and show its hint whenever the value is empty.
- Reference an `alertdialog`'s message with `aria-describedby`.
- Label images of math with the expression as it would be spoken.
- Reference each `figure` from the main text.
- Make the element referenced by `aria-details` visible to all users.

([aria-labelledby](https://www.w3.org/TR/wai-aria-1.2/#aria-labelledby))

Automation: Placeholder display needs interaction, and label quality, spoken math labels and figure references need manual or AI review.

#### DON'T

```html
<h2>Shipping address</h2>
<div role="region" aria-label="Section 2">...</div>
<div contenteditable role="searchbox" aria-placeholder="MM-DD-YYYY"></div>
```

#### DO

```html
<div role="region" aria-labelledby="shipping-h">
  <h2 id="shipping-h">Shipping address</h2>
  ...
</div>
<span id="label">Birthday:</span>
<div contenteditable role="searchbox" aria-labelledby="label" aria-placeholder="MM-DD-YYYY"></div>
```

---

### ARIA-USAGE004 - Make relationships explicit where the DOM doesn't show them

Make relationships explicit where the DOM doesn't show them:
- Group `radio` elements for one value in a `radiogroup` (or own them with `aria-owns`).
- Associate each `tabpanel` with its tab (`aria-controls` on the tab or `aria-labelledby` on the panel).
- Identify a `definition`'s term with role `term`, referenced with `aria-labelledby` or nested inside it.
- Reference a `gridcell`'s headers with `aria-describedby` when the DOM doesn't show them.
- Reference the container of an `aria-expanded` element with `aria-controls` when it doesn't own it.
- Don't use `aria-owns` for a relationship the DOM already shows.

([aria-owns](https://www.w3.org/TR/wai-aria-1.2/#aria-owns))

Automation: Whether gridcell headers can be derived from the DOM needs manual or AI review.

Prevailing rule: H71 in wcag2.2-rules.md (see C10 in [overlap.md](./overlap.md#conflicts)). The `radiogroup` requirement applies to explicit `role="radio"` elements only; native radio buttons grouped with `fieldset`/`legend` pass.

#### DON'T

```html
<div role="tablist" aria-label="Account">
  <button role="tab" aria-selected="true">Profile</button>
</div>
<div role="tabpanel">...</div>
<button aria-expanded="false">Filters</button>
<div id="filters" hidden>...</div>
```

#### DO

```html
<div role="tablist" aria-label="Account">
  <button role="tab" id="tab-profile" aria-selected="true" aria-controls="panel-profile">Profile</button>
</div>
<div role="tabpanel" id="panel-profile" aria-labelledby="tab-profile">...</div>
<button aria-expanded="false" aria-controls="filters">Filters</button>
<div id="filters" hidden>...</div>
```

---

## Deprecated Features

### ARIA-DEPR001 - Don't use the deprecated directory role

Don't use the deprecated `directory` role; use `list` or a native list instead. ([directory](https://www.w3.org/TR/wai-aria-1.2/#directory))

#### DON'T

```html
<ul role="directory">
  <li><a href="#intro">Introduction</a></li>
</ul>
```

#### DO

```html
<ul>
  <li><a href="#intro">Introduction</a></li>
</ul>
```

---

### ARIA-DEPR002 - Don't use the deprecated aria-grabbed and aria-dropeffect attributes

Don't use the deprecated `aria-grabbed` and `aria-dropeffect`. Where they remain, update the drop targets' `aria-dropeffect` while an element is grabbed (back to `none` afterwards) and show the drop targets visually. ([aria-grabbed](https://www.w3.org/TR/wai-aria-1.2/#aria-grabbed))

Automation: The drop-target behavior needs JS execution and interaction.

#### DON'T

```html
<li draggable="true" aria-grabbed="false">Task: write report</li>
<ul aria-dropeffect="move" aria-label="Done">...</ul>
```

#### DO

```html
<li draggable="true">
  Task: write report
  <button type="button">Move to Done</button>
</li>
```

---

### ARIA-DEPR003 - Don't use aria-disabled, aria-errormessage, aria-haspopup or aria-invalid on roles that don't support them

Don't use `aria-disabled`, `aria-errormessage`, `aria-haspopup` or `aria-invalid` on roles outside the supported lists below; their global use is deprecated in 1.2. ([aria-disabled](https://www.w3.org/TR/wai-aria-1.2/#aria-disabled))

| Attribute | Roles that support it |
|---|---|
| `aria-disabled` | application, button, checkbox, columnheader, combobox, grid, gridcell, group, link, listbox, menu, menubar, menuitem, menuitemcheckbox, menuitemradio, option, radio, radiogroup, row, rowheader, scrollbar, searchbox, separator, slider, spinbutton, switch, tab, tablist, textbox, toolbar, tree, treegrid, treeitem |
| `aria-errormessage`, `aria-invalid` | application, checkbox, columnheader, combobox, gridcell, listbox, radiogroup, rowheader, searchbox, slider, spinbutton, switch, textbox, tree, treegrid |
| `aria-haspopup` | application, button, columnheader, combobox, gridcell, link, menuitem, menuitemcheckbox, menuitemradio, rowheader, searchbox, slider, tab, textbox, treeitem |

#### DON'T

```html
<div role="img" aria-label="Profile photo" aria-haspopup="menu"></div>
<section aria-disabled="true">...</section>
```

#### DO

```html
<button aria-haspopup="menu" aria-label="Profile photo options">
  <img src="me.jpg" alt="">
</button>
<fieldset disabled>...</fieldset>
```

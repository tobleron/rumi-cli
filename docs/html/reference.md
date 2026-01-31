# HTML Quick Reference for Rumi-CLI

## Semantic Structure
- `<main>`, `<header>`, `<footer>`, `<nav>`, `<article>`, `<section>`
- `<aside>` for sidebars

## Forms
```html
<form onsubmit="handleSubmit()">
  <label for="name">Name:</label>
  <input type="text" id="name" name="name" required />
  <button type="submit">Submit</button>
</form>
```

## Inputs
- Types: `text`, `password`, `email`, `number`, `checkbox`, `radio`
- Attributes: `placeholder`, `disabled`, `readonly`, `value`

## Modern Elements
- `<dialog>` for modals
- `<details>` & `<summary>` for accordions
- `<picture>` for responsive images

## Accessibility
- Always use `alt` for `<img>`.
- Use `aria-label` if no text content.
- Ensure heading hierarchy (`h1` -> `h2`).

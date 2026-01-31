# CSS Quick Reference for Rumi-CLI

## Layout (Flexbox)
- `display: flex;`
- `justify-content: center | space-between | flex-start;`
- `align-items: center | stretch;`
- `flex-direction: row | column;`

## Layout (Grid)
- `display: grid;`
- `grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));`
- `gap: 1rem;`

## Selectors
- Class: `.classname`
- ID: `#idname`
- Child: `parent > child`
- State: `:hover`, `:active`, `:focus`

## Modern Features
- Variables: `--main-color: #333;` -> `color: var(--main-color);`
- Calc: `width: calc(100% - 20px);`
- Media Queries: `@media (max-width: 768px) { ... }`

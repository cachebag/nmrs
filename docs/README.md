# nmrs Documentation

This directory contains the mdBook-based user guide for nmrs.

## Building Locally

Install mdBook:

```bash
cargo install mdbook
```

Build the book:

```bash
cd docs
mdbook build
```

Serve with live reload:

```bash
mdbook serve --open
```

The documentation will be available at http://localhost:3000

## Structure

- `src/` - Markdown source files
- `book.toml` - mdBook configuration
- `theme/` - Custom CSS and styling
- `book/` - Generated output (gitignored)

## Contributing

When adding new pages:

1. Create the markdown file in the appropriate `src/` subdirectory
2. Add it to `src/SUMMARY.md` to include it in the table of contents
3. Build and preview locally to ensure it looks correct

## Deployment

The documentation is automatically built and deployed to GitHub Pages via the `.github/workflows/docs.yml` workflow on every push to `master`.

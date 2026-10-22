# Customization Guide

Hakanai supports customizing the web interface through custom assets, allowing you to white-label the application or match your organization's branding.

## Custom Assets System

Configure custom assets using the `--custom-assets-dir` option:

```bash
# Start server with custom assets
hakanai-server --custom-assets-dir /path/to/custom/assets

# Or using environment variable
HAKANAI_CUSTOM_ASSETS_DIR=/path/to/custom/assets hakanai-server
```

## Supported Assets

Place custom files in your custom directory using these exact filenames:

```
/path/to/custom/assets/
├── style.css        # Additional CSS (appended to default styles)
├── logo.svg         # Main logo (overrides default)
├── icon.svg         # Browser favicon (overrides default)
├── banner.svg       # Banner for dark mode (overrides default)
├── banner-light.svg # Banner for light mode (overrides default)
```

## Asset Types

### CSS Customization (`style.css`)
Custom CSS is **appended** to the default styles, allowing you to extend and modify the appearance:

**Behavior**: Appended to existing styles (not replaced)

### Image Assets (`logo.svg`, `icon.svg`, `banner.svg`, `banner-light.svg`)
Custom images completely **override** the default assets.

**Behavior**: Replaces default images entirely

## How Asset Customization Works

- **CSS (`style.css`)**: Your custom CSS is **appended** after the default styles, allowing you to override specific properties while keeping the base styling intact
- **Images (`*.svg`)**: Your custom images **replace** the default images completely

```css
/* Example custom styles */
:root {
  --primary-color: #your-brand-color;
  --background-color: #your-bg-color;
}

.header {
  background: var(--primary-color);
}

.logo {
  max-height: 60px;
}

/* Hide elements if needed */
.footer {
  display: none;
}
```

## Performance

- **Caching**: All assets are cached in memory after first load
- **File validation**: Only whitelisted filenames are loaded for security

## Deployment Examples

### Docker with Volume Mount
```bash
docker run -d \
  -p 8080:8080 \
  -v /host/custom/assets:/app/assets:ro \
  -e HAKANAI_CUSTOM_ASSETS_DIR=/app/assets \
  ghcr.io/czerwonk/hakanai:latest
```

### Docker Compose
```yaml
services:
  hakanai:
    image: ghcr.io/czerwonk/hakanai:latest
    ports:
      - "8080:8080"
    volumes:
      - ./custom-assets:/app/assets:ro
    environment:
      - HAKANAI_CUSTOM_ASSETS_DIR=/app/assets
```

## Theme Variables

The default theme ("sumi & washi") uses colors from the [Kanagawa](https://github.com/rebelot/kanagawa.nvim) palette: wave/dragon for dark mode and lotus for light mode. Each color is defined once per mode (`--dark-*` / `--light-*`) and exposed to components through `--theme-*` pointers that switch with the active theme. To re-brand, override the per-mode values:

```css
:root {
  /* Dark mode */
  --dark-primary: #7fb4ca;        /* links, focus rings, active states */
  --dark-primary-hover: #a3d4d5;
  --dark-bg-main: #16161d;        /* page background */
  --dark-bg-container: #1f1f28;   /* card */
  --dark-bg-input: #1a1a22;       /* form fields */
  --dark-bg-hover: #2a2a37;
  --dark-text: #dcd7ba;
  --dark-text-secondary: #c8c093;
  --dark-text-muted: #9e9b93;
  --dark-border: #363646;
  --dark-border-strong: #54546d;
  --dark-button-bg: #2d4f67;      /* primary action (indigo) */
  --dark-button-text: #dcd7ba;
  --dark-accent: #d27e99;         /* sakura, used sparingly */
  --dark-success: #98bb6c;
  --dark-warning: #e6c384;
  --dark-error: #e46876;
  --dark-glow-1: #223249;         /* background glow at the top */
  --dark-glow-2: #43242b;         /* background glow bottom right */

  /* Light mode: same names with the --light- prefix */
  --light-primary: #2d4f67;
  --light-bg-main: #8899a5;
  --light-bg-container: #b8c3cb;
  --light-button-bg: #2d4f67;
  /* ... */

  /* Spacing */
  --spacing-xs: 0.5rem;
  --spacing-sm: 0.75rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;

  /* Shape */
  --border-radius-sm: 0.25rem;
  --border-radius: 0.5rem;
  --border-radius-lg: 0.875rem;
}
```

## Security Notes

- Custom directory should be read-only for the hakanai process
- Only files with whitelisted names are loaded
- File contents are not validated - ensure CSS/SVG are safe
- Assets are cached permanently until server restart

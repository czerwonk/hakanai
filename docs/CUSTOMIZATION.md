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
├── banner.svg       # Banner (overrides default)
```

## Asset Types

### CSS Customization (`style.css`)
Custom CSS is **appended** to the default styles, allowing you to extend and modify the appearance:

**Behavior**: Appended to existing styles (not replaced)

### Image Assets (`logo.svg`, `icon.svg`, `banner.svg`)
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

The default CSS includes these CSS custom properties you can customize:

```css
:root {
  /* Colors */
  --primary-color: #007bff;
  --success-color: #28a745;
  --danger-color: #dc3545;
  --warning-color: #ffc107;
  
  /* Backgrounds */
  --background-color: #ffffff;
  --surface-color: #f8f9fa;
  --border-color: #dee2e6;
  
  /* Text */
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --text-muted: #868e96;
  
  /* Spacing */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 3rem;
}
```

## Security Notes

- Custom directory should be read-only for the hakanai process
- Only files with whitelisted names are loaded
- File contents are not validated - ensure CSS/SVG are safe
- Assets are cached permanently until server restart

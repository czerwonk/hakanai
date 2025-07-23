# Hakanai Shortcuts Integration

This repository includes an Apple Shortcuts file (`share.shortcut`) that enables quick sharing of secrets through Hakanai using a clipboard-based workflow.

## How It Works

The shortcut uses a **clipboard-based workflow** that requires user interaction:

1. **Copy your content** to the clipboard first (text, images, files, etc.)
2. **Run the shortcut** - it will:
   - Read the content from your clipboard
   - Base64-encode the content
   - Send it to the Hakanai server
   - Copy the resulting secret link to your clipboard
3. **Paste the link** wherever you need it

## Installation

1. Download `share.shortcut` to your iOS/macOS device
2. Open the file - it will automatically import into the Shortcuts app
3. (Optional) If your Hakanai instance requires authentication:
   - Edit the shortcut
   - Find the empty token field
   - Add your Hakanai token

## Usage

### Basic Workflow
1. **Copy** the content you want to share (Cmd+C / long-press → Copy)
2. **Run** the Hakanai shortcut from:
   - Shortcuts app
   - Widget
   - Share sheet (select "Hakanai Share")
   - Siri ("Run Hakanai Share")
3. **Wait** for the success notification
4. **Paste** the secret link from your clipboard

### Example Use Cases
- **Text**: Copy text → Run shortcut → Share link
- **Images**: Copy image → Run shortcut → Share link  
- **Files**: Copy file → Run shortcut → Share link
- **Code snippets**: Copy code → Run shortcut → Share link

## Why Clipboard?

The clipboard-based approach provides:
- **Universal compatibility**: Works with any app that supports copy/paste
- **User control**: You explicitly choose what to share
- **Security**: No automatic access to your data without your action
- **Flexibility**: Share content from any source

## Configuration

### Server URL
Default: `https://hakanai.link/share`

To use a different server:
1. Edit the shortcut
2. Find the URL action
3. Replace with your Hakanai instance URL

### Authentication Token
The token field is empty by default. If your server requires authentication:
1. Generate a token: `hakanai token --limit 5m --ttl 30d`
2. Edit the shortcut
3. Add your token
4. Save changes

## Technical Details

The shortcut performs these actions:
1. Gets current clipboard content
2. Base64-encodes the data
3. Creates JSON payload: `{"data": "<base64>", "filename": "", "token": ""}`
4. POST request to `/share` endpoint
5. Extracts link from response
6. Sets clipboard to the secret link

## Security Notes

- Content is only accessed when you explicitly run the shortcut
- All encryption happens on-device before transmission
- The server never sees plaintext data
- Links are one-time use only
- Clear sensitive data from clipboard after sharing

## Troubleshooting

**"Clipboard is empty"**: Copy content before running the shortcut

**"Unauthorized"**: Add your authentication token to the shortcut

**"Size limit exceeded"**: Content too large for your token/anonymous limits

**No notification**: Check Shortcuts app permissions in Settings

## Tips

- Add the shortcut to your home screen for quick access
- Use Siri: "Hey Siri, run Hakanai Share"
- Add to Control Center for faster access
- Create automations for specific workflows
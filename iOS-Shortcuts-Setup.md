# iOS Shortcuts Integration for Hakanai

This integration allows you to share any file or text from iOS directly to your Hakanai server through the native iOS Share Sheet, without requiring an App Store app.

## Setup Instructions

### 1. Create the Shortcut Manually

Due to iOS 15+ security requirements, shortcuts must be created manually. Follow these steps:

1. Open the **Shortcuts** app on iOS
2. Tap **"+"** to create a new shortcut
3. Follow the detailed configuration steps below

### 2. Shortcut Configuration Steps

**Step 1: Basic Setup**
1. Name your shortcut: **"Share with Hakanai"**
2. Tap the settings icon (⚙️) and configure:
   - **Use with Share Sheet**: Enable
   - **Accepted Types**: Select "Anything" (or specific types if preferred)

**Step 2: Add Actions**
Add these actions in order:

1. **Ask for Input** (Text)
   - Prompt: "Enter your Hakanai server URL"
   - Default Answer: `https://your-hakanai-server.com`
   - Input Type: URL

2. **Set Variable**
   - Variable Name: `ServerURL`

3. **Ask for Input** (Text) 
   - Prompt: "Enter your auth token (leave empty if anonymous)"
   - Default Answer: (empty)
   - Input Type: Text

4. **Set Variable**
   - Variable Name: `AuthToken`

5. **Get Contents of URL**
   - URL: `[ServerURL]/ios-shortcuts`
   - Method: GET

6. **Set Variable**
   - Variable Name: `HakanaiClient`

7. **Get Name** (from Shortcut Input)
   - Set Variable: `FileName`

8. **Base64 Encode** (Shortcut Input)
   - Set Variable: `FileData`

9. **Run JavaScript on Web Page**
   - JavaScript: See configuration template below
   - Web Page: Data URL with HakanaiClient content

10. **Copy to Clipboard** (JavaScript Result)

11. **Show Notification**
    - Title: "Hakanai Secret Created"
    - Body: "Secret URL copied to clipboard"

**Step 3: JavaScript Execution**
For the "Run JavaScript on Web Page" action:
- **Web Page**: Use the URL `[ServerURL]/ios-shortcuts`
- **JavaScript**: `window.HakanaiShortcuts.sendSecret(FileData, FileName, ServerURL, AuthToken)`

The `/ios-shortcuts` endpoint already contains all the necessary JavaScript client code.


## Usage

### Sharing Files
1. Open any app (Photos, Files, Mail, etc.)
2. Select a file/photo/document
3. Tap the Share button (📤)
4. Select "Share with Hakanai" from the share menu
5. The secret URL will be automatically copied to your clipboard
6. Share the URL through any messaging app

### Sharing Text
1. Select text in any app
2. Tap Share → "Share with Hakanai"
3. Secret URL copied to clipboard

## Technical Details

### Security
- **Client-side encryption**: All encryption happens on your iOS device
- **Zero-knowledge**: Your Hakanai server never sees the original content
- **AES-256-GCM**: Same encryption as the web and CLI clients
- **Secure URLs**: Decryption key is in the URL fragment (never sent to server)

### How It Works
1. **File Processing**: iOS converts the shared content to base64
2. **JavaScript Execution**: Loads your server's `hakanai-client.js` in a web view
3. **Encryption**: Uses the same crypto library as the web interface
4. **API Call**: Sends encrypted data to `/api/v1/secret` endpoint
5. **URL Generation**: Creates shareable URL with decryption key
6. **Clipboard**: Automatically copies URL for easy sharing

### File Type Support
- **Documents**: PDF, Word, Excel, PowerPoint, etc.
- **Images**: JPEG, PNG, GIF, HEIC, etc.
- **Videos**: MP4, MOV, etc.
- **Archives**: ZIP, RAR, etc.
- **Text**: Plain text, code files, etc.
- **Any file type**: The shortcut works with any file iOS can share

### Server Requirements
- Your Hakanai server must be accessible from the internet
- The `/ios-shortcuts` endpoint serves the JavaScript client
- HTTPS recommended for security
- CORS may need to be configured for cross-origin requests

## Troubleshooting

### "Shortcut Failed" Error
- Check your server URL is correct and accessible
- Verify your auth token (if using authentication)
- Ensure your server has the `/ios-shortcuts` endpoint

### "Network Error"
- Check internet connectivity
- Verify HTTPS certificate is valid
- Check if your server requires authentication

### "Permission Denied"
- Your auth token may be invalid or expired
- Check if your server allows anonymous access (if not using a token)

### File Too Large
- Check your server's upload size limit
- Anonymous users may have smaller size limits
- Consider upgrading to an authenticated token with higher limits

## Advanced Configuration

### Custom Timeout
You can modify the shortcut to change the default 24-hour expiration:
1. Edit the shortcut in Shortcuts app
2. Find the JavaScript execution step
3. Change `86400` (seconds) to your preferred duration

### Multiple Servers
Create separate shortcuts for different Hakanai servers:
1. Duplicate the shortcut
2. Rename it (e.g., "Share with Hakanai Work")
3. Configure different server URL and token

### Automation
You can trigger the shortcut programmatically:
- Use Siri: "Hey Siri, share with Hakanai"
- Create automation rules based on location, time, etc.
- Call from other shortcuts using "Run Shortcut" action

## Privacy & Security Notes

- The shortcut runs entirely on your device - no data is sent to Apple
- Server URL and auth token are stored locally in the shortcut
- All encryption happens before any network requests
- The original file never leaves your device in unencrypted form
- URLs are safe to share through any channel (email, messaging, etc.)

## Support

If you encounter issues:
1. Check your server logs for error details
2. Verify the `/ios-shortcuts` endpoint is accessible
3. Test with the web interface first to isolate issues
4. Check iOS Shortcuts app permissions and settings

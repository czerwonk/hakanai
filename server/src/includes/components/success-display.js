import { createButton, generateRandomId } from "../core/dom-utils.js";
import { copyToClipboardByElementId } from "../core/clipboard.js";
import { QRCodeGenerator } from "../core/qr-generator.js";
/**
 * Display unified success result with URL, QR code, and security note
 * @param url - The secret URL to display
 * @param options - Configuration options
 */
export function displaySuccessResult(url, options) {
    const { container } = options;
    container.className = "result success";
    container.innerHTML = "";
    // 1. Success header with instructions - ALWAYS shown
    createSuccessHeader(container);
    // 2. URL display (with separate key support)
    createUrlSection(container, url, options.separateKeyMode);
    // 3. QR Code section - ALWAYS attempted
    createQRCodeSection(container, url);
    // 4. Note section - ALWAYS shown for security
    createNoteSection(container);
}
/**
 * Create success header with title and instructions
 */
function createSuccessHeader(container) {
    var _a, _b;
    const title = document.createElement("h3");
    title.textContent =
        ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("msg.successTitle")) || "Secret Created Successfully";
    container.appendChild(title);
    const instructions = document.createElement("p");
    instructions.className = "share-instructions";
    instructions.textContent =
        ((_b = window.i18n) === null || _b === void 0 ? void 0 : _b.t("msg.shareInstructions")) ||
            "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.";
    container.appendChild(instructions);
}
/**
 * Create URL display section with copy functionality
 */
function createUrlSection(container, url, separateKeyMode) {
    const urlContainer = document.createElement("div");
    urlContainer.className = "url-container";
    if (separateKeyMode) {
        createSeparateUrlDisplay(urlContainer, url);
    }
    else {
        createCombinedUrlDisplay(urlContainer, url);
    }
    container.appendChild(urlContainer);
}
/**
 * Create combined URL display (traditional mode)
 */
function createCombinedUrlDisplay(container, url) {
    var _a, _b;
    const urlId = generateRandomId();
    const label = document.createElement("label");
    label.textContent = ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("label.secretUrl")) || "Secret URL:";
    label.setAttribute("for", urlId);
    container.appendChild(label);
    const inputContainer = document.createElement("div");
    inputContainer.className = "input-group";
    const urlInput = document.createElement("input");
    urlInput.type = "text";
    urlInput.id = urlId;
    urlInput.value = url;
    urlInput.readOnly = true;
    urlInput.className = "url-input";
    inputContainer.appendChild(urlInput);
    const copyButton = createButton("copy-button", ((_b = window.i18n) === null || _b === void 0 ? void 0 : _b.t("button.copy")) || "Copy URL", "Copy secret URL to clipboard", () => copyToClipboardByElementId(urlId, copyButton));
    inputContainer.appendChild(copyButton);
    container.appendChild(inputContainer);
}
/**
 * Create separate URL and key display (enhanced security mode)
 */
function createSeparateUrlDisplay(container, fullUrl) {
    var _a, _b, _c, _d;
    const [url, key] = fullUrl.split("#");
    const id = generateRandomId();
    const urlId = id;
    const keyId = id + "-key";
    // URL section
    const urlLabel = document.createElement("label");
    urlLabel.textContent = ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("label.secretUrl")) || "Secret URL:";
    urlLabel.setAttribute("for", urlId);
    container.appendChild(urlLabel);
    const urlInputContainer = document.createElement("div");
    urlInputContainer.className = "input-group";
    const urlInput = document.createElement("input");
    urlInput.type = "text";
    urlInput.id = urlId;
    urlInput.value = url;
    urlInput.readOnly = true;
    urlInput.className = "url-input";
    urlInputContainer.appendChild(urlInput);
    const urlCopyButton = createButton("copy-button", ((_b = window.i18n) === null || _b === void 0 ? void 0 : _b.t("button.copy")) || "Copy URL", "Copy secret URL to clipboard", () => copyToClipboardByElementId(urlInput.id, urlCopyButton));
    urlInputContainer.appendChild(urlCopyButton);
    container.appendChild(urlInputContainer);
    // Key section
    const keyLabel = document.createElement("label");
    keyLabel.textContent =
        ((_c = window.i18n) === null || _c === void 0 ? void 0 : _c.t("label.decryptionKey")) || "Decryption Key:";
    keyLabel.setAttribute("for", keyId);
    container.appendChild(keyLabel);
    const keyInputContainer = document.createElement("div");
    keyInputContainer.className = "input-group";
    const keyInput = document.createElement("input");
    keyInput.type = "text";
    keyInput.id = keyId;
    keyInput.value = key;
    keyInput.readOnly = true;
    keyInput.className = "url-input";
    keyInputContainer.appendChild(keyInput);
    const keyCopyButton = createButton("copy-button", ((_d = window.i18n) === null || _d === void 0 ? void 0 : _d.t("button.copy")) || "Copy Key", "Copy decryption key to clipboard", () => copyToClipboardByElementId(keyInput.id, keyCopyButton));
    keyInputContainer.appendChild(keyCopyButton);
    container.appendChild(keyInputContainer);
}
/**
 * Create QR code section (with graceful degradation)
 */
async function createQRCodeSection(container, url) {
    try {
        await QRCodeGenerator.ensureWasmLoaded();
        const qrSvg = QRCodeGenerator.generateQRCode(url);
        if (qrSvg) {
            const qrSection = createQRDisplayElement(qrSvg);
            container.appendChild(qrSection);
        }
    }
    catch (error) {
        // Silent graceful degradation - no UI indication needed
        // QR code simply doesn't appear if WASM fails
        console.debug("QR code not available:", error);
    }
}
/**
 * Create QR code display element
 */
function createQRDisplayElement(qrSvg) {
    var _a;
    const qrSection = document.createElement("div");
    qrSection.className = "qr-code-section";
    const qrLabel = document.createElement("label");
    qrLabel.textContent = ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("label.qrCode")) || "QR Code:";
    qrSection.appendChild(qrLabel);
    const qrContainer = document.createElement("div");
    qrContainer.className = "qr-code-container";
    qrContainer.innerHTML = qrSvg;
    qrSection.appendChild(qrContainer);
    return qrSection;
}
/**
 * Create security note section
 */
function createNoteSection(container) {
    var _a;
    const note = document.createElement("p");
    note.className = "secret-note";
    const noteText = ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("msg.createNote")) ||
        "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.";
    const colonIndex = noteText.indexOf(":");
    if (colonIndex > 0) {
        const strong = document.createElement("strong");
        strong.textContent = noteText.substring(0, colonIndex + 1);
        note.appendChild(strong);
        const remaining = document.createTextNode(noteText.substring(colonIndex + 1));
        note.appendChild(remaining);
    }
    else {
        note.textContent = noteText;
    }
    container.appendChild(note);
}

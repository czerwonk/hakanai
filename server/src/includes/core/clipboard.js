import { announceToScreenReader } from "./dom-utils.js";
// Constants
const COPY_FEEDBACK_TIMEOUT = 2000;
/**
 * Copy text to clipboard with visual feedback
 * @param text - Text to copy to clipboard
 * @param button - Button element to show feedback on
 */
export function copyToClipboard(text, button) {
    const originalText = button.textContent || "Copy";
    navigator.clipboard
        .writeText(text)
        .then(() => {
        var _a;
        return showCopySuccess(button, originalText, ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("button.copied")) || "Copied!");
    })
        .catch(() => {
        var _a;
        return showCopyFailure(button, originalText, ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("msg.copyFailed")) || "Copy Failed");
    });
}
function showCopySuccess(button, originalText, successMessage) {
    button.textContent = successMessage;
    button.classList.add("copied");
    announceToScreenReader(successMessage);
    setTimeout(() => {
        button.textContent = originalText;
        button.classList.remove("copied");
    }, COPY_FEEDBACK_TIMEOUT);
}
function showCopyFailure(button, originalText, failedMessage) {
    // Show failure state visually without disruptive alerts
    button.textContent = failedMessage || "Copy Failed";
    button.classList.add("copy-failed");
    announceToScreenReader(failedMessage || "Copy Failed");
    setTimeout(() => {
        button.textContent = originalText;
        button.classList.remove("copy-failed");
    }, COPY_FEEDBACK_TIMEOUT);
}
export function copyToClipboardByElementId(elementId, button) {
    var _a;
    const input = document.getElementById(elementId);
    if (input) {
        copyToClipboard(input.value, button);
    }
    else {
        showCopyFailure(button, button.textContent || "Copy", ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t("msg.copyFailed")) || "Copy Failed");
    }
}

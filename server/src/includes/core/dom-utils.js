// Constants
const SCREEN_READER_ANNOUNCEMENT_TIMEOUT = 1000;
/**
 * Create a button element with consistent styling and accessibility
 * @param className - CSS class for the button
 * @param text - Button text content
 * @param ariaLabel - Accessible label for screen readers
 * @param clickHandler - Click event handler
 * @returns Configured button element
 */
export function createButton(className, text, ariaLabel, clickHandler) {
    const button = document.createElement("button");
    button.className = className;
    button.type = "button";
    button.textContent = text;
    button.setAttribute("aria-label", ariaLabel);
    if (clickHandler) {
        button.addEventListener("click", clickHandler);
    }
    return button;
}
export function createButtonContainer() {
    const container = document.createElement("div");
    container.className = "buttons-container";
    return container;
}
/**
 * Securely clear sensitive input by overwriting with dummy data
 * @param input - HTML input element containing sensitive data
 */
export function secureInputClear(input) {
    if (input.value.length > 0) {
        const length = input.value.length;
        // Multiple overwrite passes
        for (let i = 0; i < 3; i++) {
            input.value = Array(length)
                .fill(0)
                .map(() => String.fromCharCode(Math.floor(Math.random() * 256)))
                .join("");
        }
        input.value = "";
    }
    // Remove from DOM for additional security
    if (input.parentNode) {
        input.parentNode.removeChild(input);
    }
}
/**
 * Announce a message to screen readers using ARIA live regions
 * @param message - Message to announce
 */
export function announceToScreenReader(message) {
    const announcement = createScreenReaderAnnouncement(message);
    document.body.appendChild(announcement);
    setTimeout(() => {
        document.body.removeChild(announcement);
    }, SCREEN_READER_ANNOUNCEMENT_TIMEOUT);
}
function createScreenReaderAnnouncement(message) {
    const announcement = document.createElement("div");
    announcement.setAttribute("role", "status");
    announcement.setAttribute("aria-live", "polite");
    announcement.className = "sr-only";
    announcement.textContent = message;
    return announcement;
}
/**
 * Create a debounced version of a function
 * @template T - Function type to debounce
 * @param func - Function to debounce
 * @param wait - Milliseconds to wait before calling
 * @returns Debounced function
 */
export function debounce(func, wait) {
    let timeout = null;
    return function executedFunction(...args) {
        if (timeout)
            clearTimeout(timeout);
        timeout = setTimeout(() => func(...args), wait);
    };
}
/**
 * Generate a unique ID to be used for dynamic elements like URL inputs
 */
export function generateRandomId() {
    return (crypto === null || crypto === void 0 ? void 0 : crypto.randomUUID) && typeof crypto.randomUUID === "function"
        ? `url-${crypto.randomUUID()}`
        : `url-${Date.now()}-${Math.random()}`;
}

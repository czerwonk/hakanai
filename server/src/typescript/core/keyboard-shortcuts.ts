// SPDX-License-Identifier: Apache-2.0

/**
 * Keyboard shortcuts for power users
 */

export interface ShortcutHandler {
  key: string;
  ctrl?: boolean;
  handler: () => void;
  description?: string;
}

export class KeyboardShortcuts {
  private shortcuts: ShortcutHandler[] = [];

  constructor() {
    this.setupListener();
  }

  /**
   * Register a keyboard shortcut
   */
  register(shortcut: ShortcutHandler): void {
    this.shortcuts.push(shortcut);
  }

  /**
   * Register multiple shortcuts at once
   */
  registerMultiple(shortcuts: ShortcutHandler[]): void {
    this.shortcuts.push(...shortcuts);
  }

  private setupListener(): void {
    document.addEventListener("keydown", (e: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in inputs, except for Ctrl combinations
      const target = e.target as HTMLElement;
      if (
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable) &&
        !e.ctrlKey
      ) {
        return;
      }

      for (const shortcut of this.shortcuts) {
        if (this.matchesShortcut(e, shortcut)) {
          e.preventDefault();
          shortcut.handler();
          break;
        }
      }
    });
  }

  private matchesShortcut(
    event: KeyboardEvent,
    shortcut: ShortcutHandler,
  ): boolean {
    return (
      event.key.toLowerCase() === shortcut.key.toLowerCase() &&
      (shortcut.ctrl === undefined || shortcut.ctrl === event.ctrlKey)
    );
  }
}

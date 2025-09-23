/**
 * FileList Component - displays a list of selected files with remove functionality
 *
 * Features:
 * - Shows file name, size, and icon
 * - Allows removing individual files
 * - Shows total size and count
 * - Indicates when files will be bundled as TAR
 */

import { formatFileSize } from "../core/formatters";
import { getFileIcon, sanitizeFileName } from "../core/file-utils";
import { showElement, hideElement } from "../core/dom-utils";
import { I18nKeys } from "../core/i18n";

export interface FileListItem {
  file: File;
  id: string;
}

export class FileListComponent {
  private container: HTMLElement;
  private files: Map<string, FileListItem> = new Map();
  private onFilesChanged: (files: File[]) => void;
  private fileListElement: HTMLElement | null = null;
  private summaryElement: HTMLElement | null = null;
  private maxTotalSize: number = 0; // 0 means no limit
  private onSizeExceeded?: (totalSize: number, maxSize: number) => void;

  constructor(
    container: HTMLElement,
    onFilesChanged: (files: File[]) => void,
    options?: {
      maxTotalSize?: number;
      onSizeExceeded?: (totalSize: number, maxSize: number) => void;
    },
  ) {
    this.container = container;
    this.onFilesChanged = onFilesChanged;
    this.maxTotalSize = options?.maxTotalSize || 0;
    this.onSizeExceeded = options?.onSizeExceeded;
    this.init();
  }

  private init(): void {
    const containerDiv = this.createContainer();

    const summaryDiv = this.createSummarySection();
    containerDiv.appendChild(summaryDiv);

    const fileList = this.createFileList();
    containerDiv.appendChild(fileList);

    this.container.appendChild(containerDiv);

    this.fileListElement = fileList;
    this.summaryElement = containerDiv;
  }

  private createContainer(): HTMLElement {
    const containerDiv = document.createElement("div");
    containerDiv.className = "file-list-container";
    hideElement(containerDiv);
    return containerDiv;
  }

  private createSummarySection(): HTMLElement {
    const summaryDiv = document.createElement("div");
    summaryDiv.className = "file-list-summary";

    const fileCount = document.createElement("span");
    fileCount.className = "file-count";

    const totalSize = document.createElement("span");
    totalSize.className = "total-size";

    const bundleIndicator = this.createBundleIndicator();

    summaryDiv.appendChild(fileCount);
    summaryDiv.appendChild(totalSize);
    summaryDiv.appendChild(bundleIndicator);

    return summaryDiv;
  }

  private createBundleIndicator(): HTMLElement {
    const bundleIndicator = document.createElement("span");
    bundleIndicator.className = "bundle-indicator";
    bundleIndicator.textContent = "📦 ";
    hideElement(bundleIndicator);

    const bundleText = document.createElement("span");
    bundleText.setAttribute("data-i18n", "fileList.willBundle");
    bundleText.textContent = "Will be bundled as TAR archive";
    bundleIndicator.appendChild(bundleText);

    return bundleIndicator;
  }

  private createFileList(): HTMLElement {
    const fileList = document.createElement("ul");
    fileList.className = "file-list";
    return fileList;
  }

  /**
   * Add files to the list - always add all files, let UI show warning
   */
  addFiles(newFiles: FileList | File[]): void {
    const filesArray = Array.from(newFiles);

    for (const file of filesArray) {
      if (this.isDuplicate(file)) {
        // skip if file already exists (by name and size)
        continue;
      }

      const id = this.generateFileId(file);
      this.files.set(id, { file, id });
    }

    this.render();
    this.notifyChange();
  }

  /**
   * Update the size limit (e.g., when auth status changes)
   */
  setMaxSize(maxSize: number): void {
    this.maxTotalSize = maxSize;
    this.render(); // Re-render to show/hide warning
  }

  /**
   * Check if current files exceed the size limit
   */
  isOverLimit(): boolean {
    return this.maxTotalSize > 0 && this.getTotalSize() > this.maxTotalSize;
  }

  /**
   * Clear all files
   */
  clear(): void {
    this.files.clear();
    this.render();
    this.notifyChange();
  }

  /**
   * Get all files as array
   */
  getFiles(): File[] {
    return Array.from(this.files.values()).map((item) => item.file);
  }

  /**
   * Get total size of all files
   */
  getTotalSize(): number {
    return this.getFiles().reduce((sum, file) => sum + file.size, 0);
  }

  /**
   * Check if we need to bundle files (more than 1)
   */
  needsBundle(): boolean {
    return this.files.size > 1;
  }

  private isDuplicate(file: File): boolean {
    for (const item of this.files.values()) {
      if (item.file.name === file.name && item.file.size === file.size) {
        return true;
      }
    }
    return false;
  }

  private generateFileId(file: File): string {
    return `file-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
  }

  private removeFile(id: string): void {
    this.files.delete(id);
    this.render();
    this.notifyChange();
  }

  private notifyChange(): void {
    this.onFilesChanged(this.getFiles());
  }

  private render(): void {
    if (!this.fileListElement || !this.summaryElement) return;

    if (this.files.size === 0) {
      hideElement(this.summaryElement);
      return;
    }

    showElement(this.summaryElement);

    this.updateSummary();
    this.renderFileList();
  }

  private updateSummary(): void {
    const countElement = this.container.querySelector(".file-count");
    const sizeElement = this.container.querySelector(".total-size");
    const bundleIndicator = this.container.querySelector(".bundle-indicator");

    if (countElement) {
      countElement.textContent = this.files.size === 1 ? "1 file selected" : `${this.files.size} files selected`;
    }

    if (sizeElement) {
      sizeElement.textContent = `(${formatFileSize(this.getTotalSize())})`;
    }

    if (bundleIndicator) {
      if (this.needsBundle()) {
        showElement(bundleIndicator as HTMLElement);
      } else {
        hideElement(bundleIndicator as HTMLElement);
      }
    }
  }

  private renderFileList(): void {
    if (!this.fileListElement) return;

    // Clear existing content
    this.clearFileList();

    // Render each file
    for (const [id, item] of this.files) {
      const li = this.createFileListItem(id, item);
      this.fileListElement.appendChild(li);
    }
  }

  private clearFileList(): void {
    if (!this.fileListElement) return;

    while (this.fileListElement.firstChild) {
      this.fileListElement.removeChild(this.fileListElement.firstChild);
    }
  }

  private createFileListItem(id: string, item: FileListItem): HTMLElement {
    const li = document.createElement("li");
    li.className = "file-list-item";
    li.dataset.fileId = id;

    const fileName = sanitizeFileName(item.file.name) || item.file.name || "";

    li.appendChild(this.createSpan(getFileIcon(fileName), "file-icon"));
    li.appendChild(this.createSpan(fileName, "file-name"));
    li.appendChild(this.createSpan(formatFileSize(item.file.size), "file-size"));
    li.appendChild(this.createRemoveButton(id, fileName));

    return li;
  }

  private createSpan(content: string, className: string): HTMLElement {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = content;

    return span;
  }

  private createRemoveButton(id: string, filename: string): HTMLElement {
    const removeBtn = document.createElement("button");
    removeBtn.className = "remove-file-btn";
    removeBtn.setAttribute("aria-label", `Remove ${filename}`);
    removeBtn.textContent = "❌";
    removeBtn.addEventListener("click", (e) => {
      e.preventDefault();
      this.removeFile(id);
    });

    return removeBtn;
  }
}

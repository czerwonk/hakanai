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
import { showElement, hideElement, generateRandomId } from "../core/dom-utils";
import { I18nKeys } from "../core/i18n";

export interface FileListItem {
  file: File;
  id: string;
}

export class FileListComponent {
  private container: HTMLElement;
  private files: Map<string, FileListItem> = new Map();
  private onFilesChanged: (files: File[]) => void;
  private containerDiv: HTMLElement | null = null;
  private fileListElement: HTMLElement | null = null;
  private summaryElement: HTMLElement | null = null;
  private warningElement: HTMLElement | null = null;
  private maxTotalSize: number | undefined = undefined;

  constructor(
    container: HTMLElement,
    onFilesChanged: (files: File[]) => void,
    options?: {
      maxTotalSize?: number;
    },
  ) {
    this.container = container;
    this.onFilesChanged = onFilesChanged;
    this.maxTotalSize = options?.maxTotalSize;
    this.init();
  }

  private init(): void {
    this.containerDiv = document.createElement("div");
    this.containerDiv.className = "file-list-container";
    hideElement(this.containerDiv);

    this.summaryElement = this.createSummarySection();
    this.containerDiv.appendChild(this.summaryElement);

    this.warningElement = document.createElement("div");
    this.warningElement.className = "limit-text danger";
    hideElement(this.warningElement);
    this.containerDiv.appendChild(this.warningElement);

    this.fileListElement = document.createElement("ul");
    this.fileListElement.className = "file-list";
    this.containerDiv.appendChild(this.fileListElement);

    this.container.appendChild(this.containerDiv);
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
    bundleText.textContent = window.i18n.t(I18nKeys.FileList.BundleNotice);
    bundleIndicator.appendChild(bundleText);

    return bundleIndicator;
  }

  /**
   * Add files to the list
   */
  addFiles(newFiles: FileList | File[]): void {
    const filesArray = Array.from(newFiles);

    for (const file of filesArray) {
      if (this.isDuplicate(file)) {
        // skip if file already exists
        continue;
      }

      const id = generateRandomId();
      this.files.set(id, { file, id });
    }

    this.render();
    this.notifyChange();
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

  private removeFile(id: string): void {
    this.files.delete(id);
    this.render();
    this.notifyChange();
  }

  private notifyChange(): void {
    this.onFilesChanged(this.getFiles());
  }

  private render(): void {
    if (!this.containerDiv || !this.fileListElement || !this.summaryElement) return;

    if (this.files.size === 0) {
      hideElement(this.containerDiv);
      return;
    }

    showElement(this.containerDiv);

    this.updateSummary();
    this.renderFileList();
  }

  private updateSummary(): void {
    const countElement = this.container.querySelector(".file-count");
    const sizeElement = this.container.querySelector(".total-size");
    const bundleIndicator = this.container.querySelector(".bundle-indicator");
    console.log("Updating summary:", {
      countElement,
      sizeElement,
      bundleIndicator,
      filesSize: this.files.size,
    });

    if (countElement) {
      const countText =
        this.files.size === 1
          ? window.i18n.t(I18nKeys.FileList.OneFileSelected)
          : `${this.files.size} ${window.i18n.t(I18nKeys.FileList.FilesSelected)}`;
      countElement.textContent = countText;
    }

    const totalSize = this.getTotalSize();
    if (sizeElement) {
      sizeElement.textContent = `(${formatFileSize(totalSize)})`;
    }

    this.validateFileSizeLimit(totalSize);

    if (bundleIndicator) {
      if (this.needsBundle()) {
        showElement(bundleIndicator as HTMLElement);
      } else {
        hideElement(bundleIndicator as HTMLElement);
      }
    }
  }

  private validateFileSizeLimit(totalSize: number): void {
    if (!this.warningElement) return;

    if (!this.maxTotalSize || this.maxTotalSize == 0 || totalSize <= this.maxTotalSize) {
      hideElement(this.warningElement);
      return;
    }

    this.warningElement.textContent = window.i18n.t(I18nKeys.Msg.FileSizeExceeded, {
      fileSize: formatFileSize(totalSize),
      limit: formatFileSize(this.maxTotalSize),
    });
    showElement(this.warningElement);
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
    removeBtn.textContent = "❌";
    removeBtn.addEventListener("click", (e) => {
      e.preventDefault();
      this.removeFile(id);
    });

    return removeBtn;
  }
}

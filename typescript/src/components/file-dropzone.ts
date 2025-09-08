// SPDX-License-Identifier: Apache-2.0

/**
 * File dropzone component with drag and drop support and mobile fallback
 * Maintains zero-knowledge architecture - all file processing happens client-side
 */

export interface FileDropzoneOptions {
  /** File input element to wrap with drag and drop */
  fileInput: HTMLInputElement;
  /** Container element to make droppable */
  container: HTMLElement;
}

/**
 * Detects if the current device supports drag and drop
 */
function supportsDragAndDrop(): boolean {
  // Check if we have the required APIs
  if (!("draggable" in document.createElement("div"))) {
    return false;
  }

  // Basic mobile detection - mobile devices typically don't support drag and drop
  const isMobile = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);

  if (isMobile) {
    return false;
  }

  // Check for touch-only devices (no mouse/trackpad)
  const isTouchDevice = "ontouchstart" in window || navigator.maxTouchPoints > 0;

  // If it's touch-only and mobile-like, assume no drag and drop support
  if (isTouchDevice && window.innerWidth < 768) {
    return false;
  }

  return true;
}

export class FileDropzone {
  private fileInput: HTMLInputElement;
  private container: HTMLElement;

  private dragCounter = 0;
  private isDragging = false;
  private isEnabled = true;

  // CSS classes for styling
  private static readonly CSS_CLASSES = {
    DROPZONE: "file-dropzone",
    DROPZONE_ACTIVE: "file-dropzone--active",
    DROPZONE_DRAGOVER: "file-dropzone--dragover",
    DROPZONE_DISABLED: "file-dropzone--disabled",
  } as const;

  constructor(options: FileDropzoneOptions) {
    this.fileInput = options.fileInput;
    this.container = options.container;

    this.init();
  }

  private init(): void {
    this.container.classList.add(FileDropzone.CSS_CLASSES.DROPZONE);

    this.fileInput.addEventListener("change", this.handleFileInputChange.bind(this));

    this.setupDragAndDrop();
    this.container.classList.add(FileDropzone.CSS_CLASSES.DROPZONE_ACTIVE);

    // Ensure single file selection only
    this.fileInput.removeAttribute("multiple");
  }

  private setupDragAndDrop(): void {
    // Prevent default drag behaviors on document to avoid page navigation
    document.addEventListener("dragover", this.preventDefaults.bind(this));
    document.addEventListener("drop", this.preventDefaults.bind(this));

    // Container-specific drag events
    this.container.addEventListener("dragenter", this.handleDragEnter.bind(this));
    this.container.addEventListener("dragover", this.handleDragOver.bind(this));
    this.container.addEventListener("dragleave", this.handleDragLeave.bind(this));
    this.container.addEventListener("drop", this.handleDrop.bind(this));

    // Add accessibility attributes
    this.container.setAttribute("role", "button");
    this.container.setAttribute("tabindex", "0");
    this.container.setAttribute("aria-label", "Drop files here or click to select");

    // Keyboard support for accessibility
    this.container.addEventListener("keydown", this.handleKeyDown.bind(this));
    this.container.addEventListener("click", this.handleClick.bind(this));
  }

  private preventDefaults(e: Event): void {
    e.preventDefault();
    e.stopPropagation();
  }

  private handleDragEnter(e: DragEvent): void {
    this.preventDefaults(e);

    if (!this.isEnabled) {
      return;
    }

    this.dragCounter++;

    if (!this.isDragging) {
      this.isDragging = true;
      this.container.classList.add(FileDropzone.CSS_CLASSES.DROPZONE_DRAGOVER);
    }
  }

  private handleDragOver(e: DragEvent): void {
    this.preventDefaults(e);

    if (!this.isEnabled) {
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = "none";
      }
      return;
    }

    // Always set dropEffect to allow drop (required for Firefox)
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "copy";
    }
  }

  private handleDragLeave(e: DragEvent): void {
    this.preventDefaults(e);

    this.dragCounter--;

    if (this.dragCounter <= 0) {
      this.dragCounter = 0;
      this.isDragging = false;
      this.container.classList.remove(FileDropzone.CSS_CLASSES.DROPZONE_DRAGOVER);
    }
  }

  private handleDrop(e: DragEvent): void {
    this.preventDefaults(e);

    if (!this.isEnabled) {
      return;
    }

    this.dragCounter = 0;
    this.isDragging = false;
    this.container.classList.remove(FileDropzone.CSS_CLASSES.DROPZONE_DRAGOVER);

    // Modern drag and drop using items API
    const items = e.dataTransfer?.items;
    if (items && items.length > 0) {
      const fileArray: File[] = [];
      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (item.kind === "file") {
          const file = item.getAsFile();
          if (file) {
            fileArray.push(file);
          }
        }
      }

      if (fileArray.length > 0) {
        const dt = new DataTransfer();
        fileArray.forEach((file) => dt.items.add(file));
        this.setFilesOnInput(dt.files);
        this.handleFiles(dt.files);
      }
    }
  }

  private setFilesOnInput(files: FileList): void {
    if (!this.fileInput || files.length === 0) {
      return;
    }

    // Create a new DataTransfer to set files on the input
    const dataTransfer = new DataTransfer();
    for (let i = 0; i < files.length; i++) {
      dataTransfer.items.add(files[i]);
    }
    this.fileInput.files = dataTransfer.files;

    // Trigger the change event
    const event = new Event("change", { bubbles: true });
    this.fileInput.dispatchEvent(event);
  }

  private handleKeyDown(e: KeyboardEvent): void {
    // Activate on Enter or Space
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      this.handleClick();
    }
  }

  private handleClick(): void {
    if (this.isEnabled) {
      this.fileInput.click();
    }
  }

  private handleFileInputChange(): void {
    const files = this.fileInput.files;
    if (files && files.length > 0) {
      this.handleFiles(files);
    }
  }

  private handleFiles(files: FileList): void {
    if (!this.isEnabled || files.length === 0) {
      return;
    }

    // Only process first file (single file mode)
    if (files.length > 1) {
      console.warn("Only the first file will be processed");
    }
  }

  /**
   * Enable or disable the dropzone
   */
  public setEnabled(enabled: boolean): void {
    this.isEnabled = enabled;

    if (enabled) {
      this.container.classList.remove(FileDropzone.CSS_CLASSES.DROPZONE_DISABLED);
      this.container.removeAttribute("aria-disabled");
      this.fileInput.disabled = false;
    } else {
      this.container.classList.add(FileDropzone.CSS_CLASSES.DROPZONE_DISABLED);
      this.container.setAttribute("aria-disabled", "true");
      this.fileInput.disabled = true;

      // Clear any drag state
      if (this.isDragging) {
        this.isDragging = false;
        this.dragCounter = 0;
        this.container.classList.remove(FileDropzone.CSS_CLASSES.DROPZONE_DRAGOVER);
      }
    }
  }

  /**
   * Check if drag and drop is supported on this device
   */
  public static isDragAndDropSupported(): boolean {
    return supportsDragAndDrop();
  }
}

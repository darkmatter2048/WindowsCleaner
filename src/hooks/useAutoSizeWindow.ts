import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  LogicalSize,
  currentMonitor,
  getCurrentWindow,
} from "@tauri-apps/api/window";

const RESIZE_MARGIN = 8;
const SCREEN_MARGIN = 32;
const BASE_WIDTH = 320;
const BASE_HEIGHT = 400;

function pixels(value: string): number {
  return Number.parseFloat(value) || 0;
}

function contentWidth(element: Element): number {
  const html = element as HTMLElement;
  return Math.max(html.scrollWidth, html.getBoundingClientRect().width);
}

function contentHeight(element: Element): number {
  const html = element as HTMLElement;
  return Math.max(html.scrollHeight, html.getBoundingClientRect().height);
}

function rowWidth(element: HTMLElement, widthOfChild = contentWidth): number {
  const style = getComputedStyle(element);
  const children = Array.from(element.children).filter((child) => {
    return getComputedStyle(child).display !== "none";
  });
  const childWidth = children.reduce((total, child) => total + widthOfChild(child), 0);
  return childWidth
    + Math.max(0, children.length - 1) * pixels(style.columnGap)
    + pixels(style.paddingLeft) + pixels(style.paddingRight)
    + pixels(style.borderLeftWidth) + pixels(style.borderRightWidth);
}

function rowHeight(element: HTMLElement): number {
  const style = getComputedStyle(element);
  return Math.max(...Array.from(element.children).map(contentHeight), 0)
    + pixels(style.paddingTop) + pixels(style.paddingBottom)
    + pixels(style.borderTopWidth) + pixels(style.borderBottomWidth);
}

function measureContent(): { width: number; height: number } | null {
  const root = document.querySelector<HTMLElement>("[data-auto-size-root]");
  const main = root?.querySelector<HTMLElement>("[data-auto-size-main]");
  const titlebar = root?.querySelector<HTMLElement>("[data-auto-size-titlebar]");
  const brand = titlebar?.querySelector<HTMLElement>("[data-auto-size-brand]");
  const footer = root?.querySelector<HTMLElement>("[data-auto-size-footer]");
  const disk = main?.querySelector<HTMLElement>("[data-auto-size-disk]");
  const actions = main?.querySelector<HTMLElement>("[data-auto-size-actions]");
  if (!root || !main || !titlebar || !brand || !footer || !disk || !actions) {
    return null;
  }

  const titlebarWidth = rowWidth(titlebar, (child) =>
    child === brand ? rowWidth(brand) : contentWidth(child));
  const diskWidth = rowWidth(disk);
  const actionStyle = getComputedStyle(actions);
  const actionWidth = Math.max(
    ...Array.from(actions.querySelectorAll("button")).map(contentWidth),
    0,
  ) + pixels(actionStyle.paddingLeft) + pixels(actionStyle.paddingRight);
  const buttonRow = actions.firstElementChild as HTMLElement | null;
  const rowStyle = buttonRow && getComputedStyle(buttonRow);
  const buttons = buttonRow
    ? Array.from(buttonRow.children).filter((child) => child.tagName === "BUTTON")
    : [];
  const actionHeight = buttons.reduce((total, button) => total + contentHeight(button), 0)
    + Math.max(0, buttons.length - 1) * pixels(rowStyle?.rowGap ?? "0")
    + pixels(actionStyle.paddingTop) + pixels(actionStyle.paddingBottom);
  const footerWidth = rowWidth(footer);
  const mainStyle = getComputedStyle(main);
  const mainHeight = rowHeight(disk)
    + actionHeight
    + pixels(mainStyle.rowGap)
    + pixels(mainStyle.paddingTop) + pixels(mainStyle.paddingBottom);

  return {
    width: Math.ceil(Math.max(BASE_WIDTH, titlebarWidth, diskWidth, actionWidth, footerWidth)),
    height: Math.ceil(Math.max(
      BASE_HEIGHT,
      titlebar.getBoundingClientRect().height + mainHeight + footer.getBoundingClientRect().height,
    )),
  };
}

/**
 * Enlarges the main window when Windows text scaling makes rendered content
 * overflow its original 320 x 400 logical-pixel layout.
 */
export function useAutoSizeWindow(enabled = true): void {
  useEffect(() => {
    if (!enabled) return;

    const appWindow = getCurrentWindow();
    let frame = 0;
    let resizing = false;
    let disposed = false;

    const resizeToContent = async () => {
      if (resizing || disposed) return;

      const content = measureContent();
      if (!content) return;

      resizing = true;
      try {
        let textScale = 1;
        try {
          textScale = await invoke<number>("get_windows_text_scale_factor");
        } catch (error) {
          console.warn("Could not read Windows text scale:", error);
        }
        const minWidth = Math.ceil(BASE_WIDTH * textScale);
        const minHeight = Math.ceil(BASE_HEIGHT * textScale);
        let workArea: LogicalSize | undefined;
        try {
          const monitor = await currentMonitor();
          workArea = monitor?.workArea.size.toLogical(monitor.scaleFactor);
        } catch (error) {
          console.warn("Could not read monitor work area:", error);
        }
        const maxWidth = workArea
          ? Math.max(BASE_WIDTH, workArea.width - SCREEN_MARGIN)
          : Math.max(BASE_WIDTH, window.screen.availWidth - SCREEN_MARGIN);
        const maxHeight = workArea
          ? Math.max(BASE_HEIGHT, workArea.height - SCREEN_MARGIN)
          : Math.max(BASE_HEIGHT, window.screen.availHeight - SCREEN_MARGIN);

        const width = Math.min(
          maxWidth,
          Math.max(
            minWidth,
            content.width > BASE_WIDTH ? content.width + RESIZE_MARGIN : BASE_WIDTH,
          ),
        );
        const height = Math.min(
          maxHeight,
          Math.max(
            minHeight,
            content.height > BASE_HEIGHT ? content.height + RESIZE_MARGIN : BASE_HEIGHT,
          ),
        );

        if (Math.abs(width - window.innerWidth) < 2
          && Math.abs(height - window.innerHeight) < 2) return;

        await appWindow.setSize(new LogicalSize(width, height));
        await appWindow.center().catch((error) => {
          console.warn("Could not center resized window:", error);
        });
      } catch (error) {
        console.error("Could not resize window for larger text:", error);
      } finally {
        resizing = false;
      }
    };

    const scheduleResize = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        void resizeToContent();
      });
    };

    const observer = new ResizeObserver(scheduleResize);
    observer.observe(document.documentElement);
    document.querySelectorAll<HTMLElement>("body *").forEach((element) => {
      observer.observe(element);
    });

    const mutationObserver = new MutationObserver(() => {
      document.querySelectorAll<HTMLElement>("body *").forEach((element) => {
        observer.observe(element);
      });
      scheduleResize();
    });
    mutationObserver.observe(document.body, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    window.addEventListener("resize", scheduleResize);
    const textScalePoll = window.setInterval(scheduleResize, 3000);
    void document.fonts?.ready.then(scheduleResize);
    scheduleResize();

    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      observer.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener("resize", scheduleResize);
      window.clearInterval(textScalePoll);
    };
  }, [enabled]);
}

// data-part for the parts inside the rich-text editor. The editor widget does not pass a data-part
// through to its textbox, toolbar buttons, mention list or mention chips, so the screen tags them
// from the DOM after each change inside the editor (and on focus, which is when a toolbar shows).
import { useEffect, type RefObject } from "react";

const P = "app.taskinspector.";

/** The editor's toolbar buttons, by the data-slot the widget gives them. */
const TOOLBAR: Record<string, string> = {
  bold: `${P}noteBold`,
  italic: `${P}noteItalic`,
  underline: `${P}noteUnderline`,
  strikethrough: `${P}noteStrike`,
  bulletList: `${P}noteBullet`,
  orderedList: `${P}noteNumbered`,
  link: `${P}noteLink`,
};

/** Gives the full part name `part` to `el` only: any other element in `root` that carries it loses it. */
function claim(root: HTMLElement, part: string, el: Element | null | undefined) {
  for (const old of root.querySelectorAll(`[data-part="${part}"]`)) {
    if (old !== el) old.removeAttribute("data-part");
  }
  el?.setAttribute("data-part", part);
}

/** Tags one note editor. `part` is the textbox's full part name (noteBody, noteBodySecond or newNoteEditor). */
export function noteTagger(part: string) {
  return (root: HTMLElement) => {
    const focused = root.contains(document.activeElement);
    const pick = (selector: string) => (focused ? root.querySelector(selector) : null);
    claim(root, part, root.querySelector('[role="textbox"]'));
    claim(root, `${P}mentionChip`, root.querySelector('[data-slot="mentionChip"]'));
    for (const [slot, name] of Object.entries(TOOLBAR)) {
      claim(root, name, pick(`[data-slot="${slot}"]`));
    }
    claim(root, `${P}mentionPopup`, pick('[data-slot="mentionList"]'));
    claim(root, `${P}mentionItem`, pick('[data-slot="mentionOption"]'));
  };
}

/** Runs `tag` on the element now, after each change inside it, and when focus moves in or out. */
export function useDomParts(ref: RefObject<HTMLElement | null>, tag: (root: HTMLElement) => void) {
  useEffect(() => {
    const root = ref.current;
    if (!root) return;
    const run = () => tag(root);
    const later = () => window.setTimeout(run, 0);
    run();
    const observer = new MutationObserver(run);
    observer.observe(root, { subtree: true, childList: true, attributes: true, attributeFilter: ["data-slot"] });
    root.addEventListener("focusin", run);
    root.addEventListener("focusout", later);
    return () => {
      observer.disconnect();
      root.removeEventListener("focusin", run);
      root.removeEventListener("focusout", later);
    };
  }, [ref, tag]);
}

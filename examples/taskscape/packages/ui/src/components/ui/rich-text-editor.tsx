import * as React from "react";
import {
  EditorContent,
  Mark,
  Node as TiptapNode,
  ReactNodeViewRenderer,
  useEditor,
  type Editor,
} from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import CharacterCount from "@tiptap/extension-character-count";
import { cva } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { Toolbar, ToolbarItem } from "./toolbar";
import { Toggle } from "./toggle";
import { Button } from "./button";
import { MentionChip } from "./mention-chip";
import { CharacterCounter } from "./character-counter";

/**
 * The one rich-text body. TipTap is the editor core; the formatting bar is the composed Radix
 * toolbar of Toggle buttons. Sizes are the spec's, at 1rem = 14px. Marks are not kept in `value`:
 * the value is the plain text, paragraphs joined by a newline, and a mention as "@name".
 */

export interface RichTextMention {
  id: string;
  name: string;
}

/** Root: the group the floating bar keys its focus state on. */
export const richTextEditorVariants = cva("group/rte relative flex min-w-0 flex-col text-content-1");

/**
 * The formatting bar. Floating hangs under the box and shows on focus within the widget (it stays
 * inside the widget's own box, so nothing above the editor is clipped); docked is always shown above the text.
 */
export const richTextBarVariants = cva("", {
  variants: {
    toolbar: {
      floating:
        "absolute left-0 top-full z-(--z-index-popover) mt-1 gap-0.5 rounded-(--radius-control) border border-edge-2 bg-surface-3 p-0.5 opacity-0 scale-95 pointer-events-none shadow-menu transition-[opacity,scale] duration-(--duration-fast) ease-standard group-focus-within/rte:opacity-100 group-focus-within/rte:scale-100 group-focus-within/rte:pointer-events-auto",
      docked: "gap-1 border-t border-edge-1 px-2 py-1.5",
    },
  },
  defaultVariants: { toolbar: "floating" },
});

/** The editable box is the ProseMirror element itself, so its size is the editor part's size. */
export const richTextFieldVariants = cva(
  [
    "relative min-w-0",
    "[&_li]:my-0 [&_ol]:list-decimal [&_ol]:pl-[1.25rem] [&_p]:m-0 [&_ul]:list-disc [&_ul]:pl-[1.25rem]",
    "[&>.ProseMirror]:box-border [&>.ProseMirror]:w-full [&>.ProseMirror]:min-w-0 [&>.ProseMirror]:break-words",
    "[&>.ProseMirror]:overflow-y-auto [&>.ProseMirror]:rounded-(--radius-field) [&>.ProseMirror]:border [&>.ProseMirror]:border-edge-2",
    "[&>.ProseMirror]:bg-surface-3 [&>.ProseMirror]:px-2 [&>.ProseMirror]:py-1.5 [&>.ProseMirror]:text-content-1 [&>.ProseMirror]:outline-none",
    "[&>.ProseMirror]:transition-[border-color,box-shadow] [&>.ProseMirror]:duration-(--duration-fast) [&>.ProseMirror]:ease-standard",
    "[&>.ProseMirror:focus]:border-focus-1 [&>.ProseMirror:focus]:ring-1 [&>.ProseMirror:focus]:ring-focus-1",
  ],
  {
    variants: {
      size: {
        sm: "[&>.ProseMirror]:min-h-[3.5rem] [&>.ProseMirror]:text-[0.9643rem] [&>.ProseMirror]:leading-[1.25rem]",
        md: "[&>.ProseMirror]:min-h-[6rem] [&>.ProseMirror]:text-[0.9643rem] [&>.ProseMirror]:leading-[1.25rem]",
        capped:
          "[&>.ProseMirror]:min-h-[4rem] [&>.ProseMirror]:max-h-[10rem] [&>.ProseMirror]:text-[0.9643rem] [&>.ProseMirror]:leading-[1.4286rem]",
      },
    },
    defaultVariants: { size: "sm" },
  },
);

/** The placeholder sits over the editor's border and padding, so it lines up with the first line. */
export const richTextPlaceholderVariants = cva(
  "pointer-events-none absolute left-px top-px right-px px-2 py-1.5 break-words text-content-3",
  {
    variants: {
      size: {
        sm: "text-[0.9643rem] leading-[1.25rem]",
        md: "text-[0.9643rem] leading-[1.25rem]",
        capped: "text-[0.9643rem] leading-[1.4286rem]",
      },
    },
    defaultVariants: { size: "sm" },
  },
);

export interface RichTextEditorProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "onChange" | "onBlur" | "defaultValue"> {
  /** The start value: plain text (newline between paragraphs), or HTML when valueFormat is "html". */
  value?: string;
  /** "text" (the default) keeps only the words; "html" keeps the marks, lists and links as HTML. */
  valueFormat?: "text" | "html";
  toolbar?: "floating" | "docked";
  size?: "sm" | "md" | "capped";
  placeholder?: string;
  autoFocus?: boolean;
  /** Characters; input stops at the limit. Read when the editor is created. */
  maxLength?: number;
  /** Mod+Enter calls onSubmit only when set. */
  submitOnModEnter?: boolean;
  /** Escape calls onCancel only when set. Unset, Escape does nothing. */
  escapeCancels?: boolean;
  /** Capture only: on blur while empty the editor collapses. */
  collapseWhenEmpty?: boolean;
  /** Attachments the @ list offers. */
  mentions?: RichTextMention[];
  disabled?: boolean;
  /** Required: the accessible name of the textbox. */
  ariaLabel?: string;
  /** value is in the editor's format; text is always the plain text (for counts and emptiness). */
  onChange?: (value: string, paragraphs: number, text: string) => void;
  onBlur?: (value: string, text: string) => void;
  onSubmit?: () => void;
  onCancel?: () => void;
  onRequestLink?: () => void;
  onDropAttachment?: (file: string) => void;
  onOpenMention?: (id: string) => void;
}

type Format = {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  bulletList: boolean;
  orderedList: boolean;
};

const NO_FORMAT: Format = {
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  bulletList: false,
  orderedList: false,
};

/** The open @ list: the typed token, the attachments that match, and the range it replaces. */
type Menu = {
  token: string;
  query: string;
  items: RichTextMention[];
  index: number;
  from: number;
  to: number;
};

type Live = Pick<
  RichTextEditorProps,
  "onChange" | "onBlur" | "onSubmit" | "onCancel" | "onRequestLink" | "onDropAttachment" | "onOpenMention"
> & {
  valueFormat: "text" | "html";
  mentions: RichTextMention[];
  submitOnModEnter: boolean;
  escapeCancels: boolean;
};

type LiveRef = { current: Live };

const NO_MENTIONS: RichTextMention[] = [];
const NO_LIVE: Live = { valueFormat: "text", mentions: NO_MENTIONS, submitOnModEnter: false, escapeCancels: false };

type PMView = Editor["view"];
type JsonNode = { type: string; text?: string; content?: JsonNode[] };

/** Plain text in, one paragraph per line out. */
function toDoc(value: string): JsonNode {
  return {
    type: "doc",
    content: value.split("\n").map((line) =>
      line === "" ? { type: "paragraph" } : { type: "paragraph", content: [{ type: "text", text: line }] },
    ),
  };
}

function serialize(ed: Editor): string {
  const { doc } = ed.state;
  return doc.textBetween(0, doc.content.size, "\n", (node) => {
    if (node.type.name === "mention") return `@${String(node.attrs.label)}`;
    return node.type.name === "hardBreak" ? "\n" : "";
  });
}

/** The editor's value in its format; an empty document is "" in both. */
function valueOf(ed: Editor, format: "text" | "html"): string {
  if (format === "text") return serialize(ed);
  return serialize(ed) === "" && ed.isEmpty ? "" : ed.getHTML();
}

function countParagraphs(ed: Editor): number {
  let count = 0;
  ed.state.doc.descendants((node) => {
    if (node.isTextblock) count++;
    return true;
  });
  return count;
}

/** Finds an "@query" before the caret. Leaves count as one character each, as the offsets need. */
function scan(
  ed: Editor,
  mentions: RichTextMention[],
  dismissed: { current: string },
  prev: Menu | null,
): Menu | null {
  const { selection } = ed.state;
  const { $from } = selection;
  if (!selection.empty || !$from.parent.isTextblock) return null;
  const before = $from.parent.textBetween(0, $from.parentOffset, undefined, "￼");
  const match = /(?:^|\s)@([^\s@]*)$/.exec(before);
  if (!match) {
    dismissed.current = "";
    return null;
  }
  const query = match[1] ?? "";
  const token = `@${query}`;
  if (token === dismissed.current) return null;
  const needle = query.toLowerCase();
  const items = mentions.filter((m) => m.name.toLowerCase().includes(needle));
  if (items.length === 0) return null;
  const at = match.index + match[0].length - token.length;
  return {
    token,
    query,
    items,
    index: prev && prev.token === token ? Math.min(prev.index, items.length - 1) : 0,
    from: $from.start() + at,
    to: $from.pos,
  };
}

function insertMention(view: PMView, menu: Menu, item: RichTextMention) {
  const node = view.state.schema.nodes.mention?.create({ id: item.id, label: item.name });
  if (!node) return;
  view.dispatch(view.state.tr.replaceWith(menu.from, menu.to, node));
  view.focus();
}

type MentionViewProps = {
  node: { attrs: Record<string, unknown>; nodeSize: number };
  getPos: () => number | undefined;
  editor: Editor;
};

/** The extensions use the live ref so their handlers see the latest props; built once per widget. */
function buildExtensions(live: LiveRef, maxLength: number) {
  const MentionView = (props: MentionViewProps) => {
    const { node, getPos, editor } = props;
    const id = String(node.attrs.id);
    const name = String(node.attrs.label || id);
    return (
      <MentionChip
        data-slot="mentionChip"
        name={name}
        onOpen={() => live.current.onOpenMention?.(id)}
        onRemove={() => {
          const pos = getPos();
          if (pos !== undefined) editor.view.dispatch(editor.state.tr.delete(pos, pos + node.nodeSize));
        }}
      />
    );
  };

  const Mention = TiptapNode.create({
    name: "mention",
    group: "inline",
    inline: true,
    atom: true,
    priority: 1000,
    addAttributes() {
      return { id: { default: "" }, label: { default: "" } };
    },
    parseHTML() {
      return [{ tag: "span[data-mention]" }];
    },
    renderHTML({ node }) {
      return ["span", { "data-mention": "", "data-id": String(node.attrs.id) }, `@${String(node.attrs.label)}`];
    },
    addNodeView() {
      return ReactNodeViewRenderer(MentionView);
    },
    addKeyboardShortcuts() {
      // A chip is one unit: Backspace right after it removes the whole chip.
      return {
        Backspace: ({ editor }) => {
          const { selection } = editor.state;
          const { $from } = selection;
          const before = $from.nodeBefore;
          if (!selection.empty || !before || before.type.name !== "mention") return false;
          editor.view.dispatch(editor.state.tr.delete($from.pos - before.nodeSize, $from.pos));
          return true;
        },
      };
    },
  });

  const Underline = Mark.create({
    name: "rteUnderline",
    parseHTML() {
      return [{ tag: "u" }];
    },
    renderHTML({ HTMLAttributes }) {
      return ["u", HTMLAttributes, 0];
    },
    addKeyboardShortcuts() {
      return { "Mod-u": () => this.editor.commands.toggleMark(this.name) };
    },
  });

  return [
    StarterKit.configure({ heading: false, blockquote: false, codeBlock: false, code: false, horizontalRule: false }),
    CharacterCount.configure({ limit: maxLength }),
    Mention,
    Underline,
  ];
}

export const RichTextEditor = React.forwardRef<HTMLDivElement, RichTextEditorProps>(function RichTextEditor(
  {
    value = "",
    valueFormat = "text",
    toolbar = "floating",
    size = "sm",
    placeholder = "",
    autoFocus = false,
    maxLength = 2000,
    submitOnModEnter = false,
    escapeCancels = false,
    collapseWhenEmpty = false,
    mentions = NO_MENTIONS,
    disabled = false,
    ariaLabel = "",
    onChange,
    onBlur,
    onSubmit,
    onCancel,
    onRequestLink,
    onDropAttachment,
    onOpenMention,
    className,
    ...props
  },
  ref,
) {
  const live = React.useRef<Live>(NO_LIVE);
  live.current = {
    valueFormat,
    mentions,
    submitOnModEnter,
    escapeCancels,
    onChange,
    onBlur,
    onSubmit,
    onCancel,
    onRequestLink,
    onDropAttachment,
    onOpenMention,
  };

  const [format, setFormat] = React.useState<Format>(NO_FORMAT);
  const [empty, setEmpty] = React.useState(true);
  const [length, setLength] = React.useState(0);
  const [menu, setMenu] = React.useState<Menu | null>(null);
  const [collapsed, setCollapsed] = React.useState(false);
  const menuRef = React.useRef<Menu | null>(null);
  const dismissed = React.useRef("");
  const [extensions] = React.useState(() => buildExtensions(live, maxLength));

  const showMenu = React.useCallback((next: Menu | null) => {
    menuRef.current = next;
    setMenu(next);
  }, []);

  /** Reads the editor after every transaction: the bar state, the placeholder, the counter and the @ list. */
  const sync = React.useCallback(
    (ed: Editor) => {
      setFormat({
        bold: ed.isActive("bold"),
        italic: ed.isActive("italic"),
        underline: ed.isActive("rteUnderline"),
        strike: ed.isActive("strike"),
        bulletList: ed.isActive("bulletList"),
        orderedList: ed.isActive("orderedList"),
      });
      setEmpty(ed.isEmpty);
      setLength(serialize(ed).length);
      showMenu(scan(ed, live.current.mentions, dismissed, menuRef.current));
    },
    [showMenu],
  );

  const handleKey = (view: PMView, event: KeyboardEvent): boolean => {
    const current = menuRef.current;
    if (current) {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        const step = event.key === "ArrowDown" ? 1 : -1;
        const index = (current.index + step + current.items.length) % current.items.length;
        showMenu({ ...current, index });
        return true;
      }
      if (event.key === "Enter" && !event.shiftKey) {
        const item = current.items[current.index];
        if (item) insertMention(view, current, item);
        showMenu(null);
        return true;
      }
      if (event.key === "Escape") {
        // Escape closes the list first; it does not cancel. Typing a new @ opens it again.
        dismissed.current = current.token;
        showMenu(null);
        return true;
      }
    }
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      if (live.current.submitOnModEnter) live.current.onSubmit?.();
      return true;
    }
    if (event.key === "Escape" && live.current.escapeCancels) {
      live.current.onCancel?.();
      return true;
    }
    return false;
  };

  const editor = useEditor({
    extensions,
    content: valueFormat === "html" ? value : toDoc(value),
    editable: !disabled,
    autofocus: autoFocus ? "end" : false,
    editorProps: {
      attributes: { role: "textbox", "aria-multiline": "true", "data-slot": "editor" },
      handleKeyDown: (view, event) => handleKey(view, event),
    },
    onUpdate: ({ editor: ed }) => {
      live.current.onChange?.(valueOf(ed, live.current.valueFormat), countParagraphs(ed), serialize(ed));
    },
    onTransaction: ({ editor: ed }) => sync(ed),
  });

  React.useEffect(() => {
    if (editor) sync(editor);
  }, [editor, sync]);

  React.useEffect(() => {
    editor?.setEditable(!disabled, false);
  }, [editor, disabled]);

  React.useEffect(() => {
    if (!editor) return;
    editor.view.dom.setAttribute("aria-label", ariaLabel);
    editor.view.dom.setAttribute("aria-disabled", disabled ? "true" : "false");
  }, [editor, ariaLabel, disabled]);

  // The value is the start value; a changed value replaces the content only when the editor is not being edited.
  React.useEffect(() => {
    if (editor && !editor.isFocused && valueOf(editor, valueFormat) !== value) {
      editor.commands.setContent(valueFormat === "html" ? value : toDoc(value), { emitUpdate: false });
    }
  }, [editor, value, valueFormat]);

  React.useEffect(() => {
    if (value !== "") setCollapsed(false);
  }, [value]);

  const act = (command: (ed: Editor) => unknown) => {
    if (editor) command(editor);
  };

  const counterNear = length > 0.9 * maxLength;
  // A hidden floating bar keeps its buttons hit-testable (the bar itself is pointer-events none until focus),
  // so a press on a button from cold still reaches it.
  const hit = toolbar === "floating" ? "pointer-events-auto" : undefined;

  return (
    <div
      ref={ref}
      className={cn(richTextEditorVariants(), className)}
      hidden={collapsed}
      {...props}
      onBlur={(event) => {
        if (event.relatedTarget instanceof Node && event.currentTarget.contains(event.relatedTarget)) return;
        showMenu(null);
        const text = editor ? serialize(editor) : "";
        live.current.onBlur?.(editor ? valueOf(editor, live.current.valueFormat) : "", text);
        if (collapseWhenEmpty && text === "") setCollapsed(true);
      }}
      onDragOverCapture={(event) => {
        if (event.dataTransfer?.types.includes("Files")) event.preventDefault();
      }}
      onDropCapture={(event) => {
        const file = event.dataTransfer?.files[0];
        if (!file || !editor || disabled) return;
        // An attachment dropped on the editor becomes a mention chip, not pasted text.
        event.preventDefault();
        event.stopPropagation();
        const node = editor.state.schema.nodes.mention?.create({ id: file.name, label: file.name });
        if (node) editor.view.dispatch(editor.state.tr.replaceSelectionWith(node));
        editor.view.focus();
        live.current.onDropAttachment?.(file.name);
      }}
    >
      <Toolbar data-slot="formatting" ariaLabel="Formatting" className={richTextBarVariants({ toolbar })}>
        <ToolbarItem asChild>
          <Toggle
            data-slot="bold"
            ariaLabel="Bold"
            icon="format_bold"
            shortcut="Meta+B"
            pressed={format.bold}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleBold().run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Toggle
            data-slot="italic"
            ariaLabel="Italic"
            icon="format_italic"
            shortcut="Meta+I"
            pressed={format.italic}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleItalic().run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Toggle
            data-slot="underline"
            ariaLabel="Underline"
            icon="format_underlined"
            shortcut="Meta+U"
            pressed={format.underline}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleMark("rteUnderline").run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Toggle
            data-slot="strikethrough"
            ariaLabel="Strikethrough"
            icon="strikethrough_s"
            pressed={format.strike}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleStrike().run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Toggle
            data-slot="bulletList"
            ariaLabel="Bulleted list"
            icon="format_list_bulleted"
            pressed={format.bulletList}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleBulletList().run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Toggle
            data-slot="orderedList"
            ariaLabel="Numbered list"
            icon="format_list_numbered"
            pressed={format.orderedList}
            disabled={disabled}
            retainFocus
            className={hit}
            onPressedChange={() => act((ed) => ed.chain().focus().toggleOrderedList().run())}
          />
        </ToolbarItem>
        <ToolbarItem asChild>
          <Button
            data-slot="link"
            ariaLabel="Link"
            icon="link"
            size="sm"
            shape="icon"
            width="content"
            iconSize={15}
            retainFocus
            className={hit}
            disabled={disabled}
            onClick={() => live.current.onRequestLink?.()}
          />
        </ToolbarItem>
      </Toolbar>

      {counterNear && (
        <CharacterCounter data-slot="counter" count={length} limit={maxLength} className="mb-0.5 self-end" />
      )}
      <div className="relative min-w-0">
        <EditorContent editor={editor} className={richTextFieldVariants({ size })} />
        {placeholder !== "" && (
          <div data-slot="placeholder" hidden={!empty} className={richTextPlaceholderVariants({ size })}>
            {placeholder}
          </div>
        )}
        <div
          data-slot="mentionList"
          role="listbox"
          aria-label="Mentions"
          hidden={menu === null}
          className={cn(
            "absolute left-0 right-0 top-full z-(--z-index-popover) rounded-(--radius-control) border border-edge-2 bg-surface-3 p-0.5 shadow-menu",
            // Under the floating bar when both show, so the two never overlap.
            toolbar === "floating" ? "mt-[3rem]" : "mt-1",
          )}
        >
          {menu &&
            menu.items.map((item, index) => (
              <div
                key={item.id}
                data-slot="mentionOption"
                role="option"
                aria-selected={index === menu.index}
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => {
                  if (editor) insertMention(editor.view, menu, item);
                  showMenu(null);
                }}
                className="select-none rounded-(--radius-field) px-2 py-1 text-[0.9643rem] leading-[1.25rem] text-content-1 aria-selected:bg-wash-2"
              >
                {item.name}
              </div>
            ))}
        </div>
      </div>

    </div>
  );
});

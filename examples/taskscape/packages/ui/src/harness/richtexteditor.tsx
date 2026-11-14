import { RichTextEditor, type RichTextMention } from "../components/ui/rich-text-editor";

/** The test sends attachments as a JSON string; the widget takes the list. */
function parseMentions(json: string): RichTextMention[] {
  if (!json) return [];
  try {
    const list: unknown = JSON.parse(json);
    if (!Array.isArray(list)) return [];
    return list.map((m: { id: unknown; name: unknown }) => ({ id: String(m.id), name: String(m.name) }));
  } catch {
    return [];
  }
}

function RichTextEditorHarness({ emit, mentions = "", ...props }: any) {
  return (
    <RichTextEditor
      {...props}
      mentions={parseMentions(mentions)}
      onChange={(value, paragraphs) => emit("changed", { value, paragraphs })}
      onBlur={(value) => emit("blurred", { value })}
      onSubmit={() => emit("submitted", {})}
      onCancel={() => emit("cancelled", {})}
      onRequestLink={() => emit("linkRequested", {})}
      onDropAttachment={(file) => emit("attachmentDropped", { file })}
      onOpenMention={(id) => emit("mentionOpened", { id })}
    />
  );
}

export default {
  id: "ui.richtexteditor",
  Component: RichTextEditorHarness,
  events: ["changed", "blurred", "submitted", "cancelled", "linkRequested", "attachmentDropped", "mentionOpened"],
};

// Note text as the editor works with it. The rich-text editor keeps marks in its own document and
// reports plain text (paragraphs on their own lines), so a note stored as HTML is shown as its text.

const MARKUP = /<\/?(p|ul|ol|li|strong|em|u|s|span|br|div|b|i)\b/i;

export function plainText(content: string): string {
  if (!MARKUP.test(content)) return content;
  const withBreaks = content
    .replace(/<\/(p|li|h[1-6]|div)>/gi, "\n")
    .replace(/<br\s*\/?>/gi, "\n");
  const doc = new DOMParser().parseFromString(withBreaks, "text/html");
  return (doc.body.textContent ?? "").replace(/\n+$/, "");
}

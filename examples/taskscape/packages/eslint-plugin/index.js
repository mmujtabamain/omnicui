// The apps may draw only with @taskscape/ui. These rules refuse what would get around it.
const RAW = new Set(["button", "a", "input", "select", "textarea", "label", "form", "dialog", "img", "svg",
  "h1", "h2", "h3", "h4", "h5", "h6", "p", "ul", "ol", "li", "table", "details", "summary", "option", "menu"]);
const PRIMITIVES = /^(@radix-ui\/|cmdk$|class-variance-authority$)/;
const RAW_COLOUR = /(^|\s)(bg|text|border|ring|fill|stroke|from|to|via|shadow)-\[(#|rgb|hsl|oklch)/;
const NAMED_COLOUR = /(^|\s)(bg|text|border|ring)-(red|blue|green|gray|slate|zinc|neutral|black|white|yellow|orange|purple|pink|sky|indigo)(-\d+)?(\s|$)/;

const noRawElements = {
  meta: { type: "problem", messages: { raw: "<{{name}}> is a raw element; use a widget from @taskscape/ui instead." } },
  create: (ctx) => ({
    JSXOpeningElement(node) {
      const name = node.name.type === "JSXIdentifier" ? node.name.name : null;
      if (name && RAW.has(name)) ctx.report({ node, messageId: "raw", data: { name } });
    },
  }),
};
const noPrimitives = {
  meta: { type: "problem", messages: { prim: "{{source}} is a UI primitive; only @taskscape/ui may use it." } },
  create: (ctx) => ({
    ImportDeclaration(node) {
      if (PRIMITIVES.test(node.source.value)) ctx.report({ node, messageId: "prim", data: { source: node.source.value } });
    },
  }),
};
const noRawStyle = {
  meta: { type: "problem", messages: { style: "No inline style: use the widget's props or a token utility.", colour: "Colours come from tokens (bg-surface-1, text-content-2…), not raw values." } },
  create: (ctx) => ({
    JSXAttribute(node) {
      if (node.name.name === "style") ctx.report({ node, messageId: "style" });
      if (node.name.name === "className" && node.value?.type === "Literal" && (RAW_COLOUR.test(node.value.value) || NAMED_COLOUR.test(node.value.value)))
        ctx.report({ node, messageId: "colour" });
    },
  }),
};

export default {
  rules: { "no-raw-elements": noRawElements, "no-ui-primitives": noPrimitives, "no-raw-style": noRawStyle },
};

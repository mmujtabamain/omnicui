import AppKit
import SwiftUI

/// Registry entries retain the rendered native view, never a second model of its bounds.
@MainActor final class PartRegistry {
    var views: [String: NSView] = [:]
    func register(_ id: String, _ view: NSView) { views[id] = view; view.setAccessibilityIdentifier(id) }
    func find(_ id: String) -> NSView? {
        if let view = views[id], view.window != nil { return view }
        if id.hasPrefix("todo.app.") {
            let name = String(id.dropFirst(9))
            for screen in ["list", "detail", "settings"] {
                if let v = views["todo.\(screen).\(name)"], v.window != nil { return v }
            }
        }
        return nil
    }
}

@MainActor final class ActionButton: NSButton {
    var invoke: (() -> Void)?
    override var acceptsFirstResponder: Bool { true }
    override func keyDown(with event: NSEvent) {
        if event.characters == " " || event.characters == "\r" { if isEnabled { performClick(nil) } }
        else { super.keyDown(with: event) }
    }
    @objc func activate() { invoke?() }
}

struct NativeButton: NSViewRepresentable {
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: ActionButton, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? nsView.fittingSize.width, height: proposal.height ?? nsView.fittingSize.height)
    }
    let id: String; let title: String; var name: String? = nil
    var color: NSColor; var onAccent: NSColor = .white; var filled = false; var plain = false; var checked: Bool? = nil; var disabled = false
    let registry: PartRegistry; let action: () -> Void
    func makeNSView(context: Context) -> ActionButton {
        let v = ActionButton(title: title, target: nil, action: #selector(ActionButton.activate))
        v.target = v; v.focusRingType = .exterior
        return v
    }
    func updateNSView(_ v: ActionButton, context: Context) {
        registry.register(id, v); v.setAccessibilityElement(true); v.setAccessibilityRole(checked == nil ? .button : .checkBox); v.title = title; v.setAccessibilityLabel(name ?? title)
        v.invoke = action; v.isEnabled = !disabled
        if let checked { v.setButtonType(.switch); v.state = checked ? .on : .off; v.title = "" }
        else { v.bezelStyle = .rounded; v.isBordered = !plain; v.bezelColor = filled ? color : nil }
        v.contentTintColor = filled ? onAccent : color
        v.font = .systemFont(ofSize: 14, weight: .medium)
        if checked == nil {
            v.attributedTitle = NSAttributedString(string: title, attributes: [.foregroundColor: filled ? onAccent : color, .font: v.font!])
        }
        if id.contains(".row") && checked == nil { v.alignment = .left }
    }
}

struct SwitchContent: View {
    let name: String
    let on: Bool
    let color: NSColor
    let action: (Bool) -> Void
    var body: some View {
        Toggle(name, isOn: Binding(get: { on }, set: action))
            .labelsHidden().toggleStyle(.switch).tint(Color(nsColor: color))
    }
}
@MainActor final class ActionSwitch: NSHostingView<SwitchContent> {
    var on = false
    var invoke: ((Bool) -> Void)?
    override var acceptsFirstResponder: Bool { true }
    override func accessibilityValue() -> Any? { on }
    override func accessibilityPerformPress() -> Bool { invoke?(!on); return true }
    override func keyDown(with event: NSEvent) {
        if event.characters == " " || event.characters == "\r" { invoke?(!on) }
        else { super.keyDown(with: event) }
    }
}
struct NativeSwitch: NSViewRepresentable {
    let id: String; let name: String; let on: Bool; let color: NSColor; let registry: PartRegistry; let action: (Bool) -> Void
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: ActionSwitch, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 52, height: proposal.height ?? 32)
    }
    func makeNSView(context: Context) -> ActionSwitch {
        ActionSwitch(rootView: SwitchContent(name: name, on: on, color: color, action: action))
    }
    func updateNSView(_ v: ActionSwitch, context: Context) {
        registry.register(id, v); v.setAccessibilityElement(true)
        v.setAccessibilityRole(NSAccessibility.Role(rawValue: "AXSwitch")); v.setAccessibilityLabel(name)
        v.on = on; v.invoke = action
        v.rootView = SwitchContent(name: name, on: on, color: color, action: action)
    }
}

/// Individual segment AX elements also serve as the keyboard focus destinations. Their frames
/// are the frames of the segment slots within the actual NSSegmentedControl.
@MainActor final class SegmentPart: NSView {
    weak var control: ActionSegments?
    var index = 0
    override var acceptsFirstResponder: Bool { true }
    override func accessibilityRole() -> NSAccessibility.Role? { .button }
    override func accessibilityLabel() -> String? { control?.label(forSegment: index) }
    override func accessibilityValue() -> Any? { control?.isSelected(forSegment: index) ?? false }
    override func accessibilityPerformPress() -> Bool { control?.choose(index); return true }
    override func keyDown(with event: NSEvent) {
        if event.characters == " " || event.characters == "\r" { control?.choose(index) }
        else { super.keyDown(with: event) }
    }
    override func mouseDown(with event: NSEvent) { window?.makeFirstResponder(self); control?.choose(index) }
    override func draw(_ dirtyRect: NSRect) {
        if window?.firstResponder === self { NSColor.keyboardFocusIndicatorColor.setStroke(); let p = NSBezierPath(roundedRect: bounds.insetBy(dx: 2, dy: 2), xRadius: 5, yRadius: 5); p.lineWidth = 2; p.stroke() }
    }
}
@MainActor final class ActionSegments: NSSegmentedControl {
    var invoke: ((Int) -> Void)?
    var parts: [SegmentPart] = []
    func choose(_ index: Int) { selectedSegment = index; invoke?(index); parts.forEach { $0.setAccessibilitySelected($0.index == index); $0.needsDisplay = true } }
    @objc func activate() { invoke?(selectedSegment) }
    override func layout() {
        super.layout()
        let width = bounds.width / CGFloat(max(1, segmentCount))
        for (i, p) in parts.enumerated() { p.frame = NSRect(x: CGFloat(i) * width, y: 0, width: width, height: bounds.height) }
    }
    override func accessibilityChildren() -> [Any]? { parts }
}
struct NativeSegments: NSViewRepresentable {
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: ActionSegments, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? nsView.fittingSize.width, height: proposal.height ?? nsView.fittingSize.height)
    }
    let ids: [String]; let labels: [String]; let selected: Int; let color: NSColor; let registry: PartRegistry; let action: (Int) -> Void
    func makeNSView(context: Context) -> ActionSegments {
        let v = ActionSegments(labels: labels, trackingMode: .selectOne, target: nil, action: #selector(ActionSegments.activate))
        v.target = v; v.segmentStyle = .rounded; v.segmentDistribution = .fillEqually
        for i in ids.indices { let p = SegmentPart(); p.index = i; p.control = v; p.setAccessibilityElement(true); v.parts.append(p); v.addSubview(p) }
        return v
    }
    func updateNSView(_ v: ActionSegments, context: Context) {
        v.selectedSegment = selected; v.selectedSegmentBezelColor = color; v.invoke = action
        for i in ids.indices { registry.register(ids[i], v.parts[i]); v.parts[i].setAccessibilitySelected(i == selected) }
    }
}

@MainActor final class EndTextField: NSTextField {
    var onChange: ((String) -> Void)?
    var onSubmit: (() -> Void)?
    override func becomeFirstResponder() -> Bool {
        let result = super.becomeFirstResponder()
        if result { currentEditor()?.selectedRange = NSRange(location: stringValue.utf16.count, length: 0) }
        return result
    }
}
struct NativeField: NSViewRepresentable {
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: EndTextField, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? nsView.fittingSize.width, height: proposal.height ?? nsView.fittingSize.height)
    }
    let id: String; let name: String; let value: String
    let color: NSColor; let registry: PartRegistry; let change: (String) -> Void; var submit: (() -> Void)? = nil
    final class Coordinator: NSObject, NSTextFieldDelegate {
        func controlTextDidChange(_ notification: Notification) { if let v = notification.object as? EndTextField { v.onChange?(v.stringValue) } }
        func control(_ control: NSControl, textView: NSTextView, doCommandBy selector: Selector) -> Bool {
            if selector == #selector(NSResponder.insertNewline(_:)), let v = control as? EndTextField, let submit = v.onSubmit { submit(); return true }
            return false
        }
    }
    func makeCoordinator() -> Coordinator { Coordinator() }
    func makeNSView(context: Context) -> EndTextField {
        let v = EndTextField(); v.delegate = context.coordinator; v.isEditable = true; v.isSelectable = true
        v.isBezeled = true; v.bezelStyle = .roundedBezel; v.drawsBackground = false; v.font = .systemFont(ofSize: 15)
        v.usesSingleLineMode = true; v.cell?.wraps = false; v.cell?.isScrollable = true
        return v
    }
    func updateNSView(_ v: EndTextField, context: Context) {
        registry.register(id, v); v.setAccessibilityElement(true); v.setAccessibilityRole(.textField); v.setAccessibilityLabel(name)
        if v.stringValue != value { v.stringValue = value }
        v.textColor = color; v.onChange = change; v.onSubmit = submit
    }
}

struct NativeLabel: NSViewRepresentable {
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSTextField, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? nsView.fittingSize.width, height: proposal.height ?? nsView.fittingSize.height)
    }
    let id: String; let text: String; let role: NSAccessibility.Role; let size: CGFloat; let color: NSColor; let registry: PartRegistry
    func makeNSView(context: Context) -> NSTextField { NSTextField(labelWithString: text) }
    func updateNSView(_ v: NSTextField, context: Context) {
        registry.register(id, v); v.setAccessibilityElement(true); v.stringValue = text; v.font = .systemFont(ofSize: size, weight: size > 18 ? .medium : .regular); v.textColor = color
        v.setAccessibilityRole(role); v.setAccessibilityLabel(text)
        if id.hasSuffix(".left") || id.hasSuffix(".empty") { v.setAccessibilityRoleDescription("status") }
    }
}

/// A transparent marker attached inside the animated SwiftUI dialog.
struct PartMarker: NSViewRepresentable {
    let id: String; let registry: PartRegistry
    func makeNSView(context: Context) -> NSView { let v = NSView(); v.setAccessibilityElement(true); v.setAccessibilityRole(.group); return v }
    func updateNSView(_ v: NSView, context: Context) { registry.register(id, v) }
}

@MainActor final class EndTextView: NSTextView {
    var onChange: ((String) -> Void)?
    override func becomeFirstResponder() -> Bool {
        let result = super.becomeFirstResponder()
        if result { setSelectedRange(NSRange(location: string.utf16.count, length: 0)) }
        return result
    }
}
struct NativeNotes: NSViewRepresentable {
    let id: String; let value: String; let color: NSColor; let registry: PartRegistry; let change: (String) -> Void
    final class Coordinator: NSObject, NSTextViewDelegate {
        func textDidChange(_ notification: Notification) {
            if let text = notification.object as? EndTextView { text.onChange?(text.string) }
        }
    }
    func makeCoordinator() -> Coordinator { Coordinator() }
    func makeNSView(context: Context) -> NSScrollView {
        let scroll = NSScrollView()
        scroll.borderType = .lineBorder; scroll.drawsBackground = false; scroll.hasVerticalScroller = true; scroll.autohidesScrollers = true
        let text = EndTextView(frame: NSRect(x: 0, y: 0, width: 558, height: 106))
        text.delegate = context.coordinator; text.isRichText = false; text.isEditable = true
        text.drawsBackground = false; text.font = .systemFont(ofSize: 15)
        text.textContainerInset = NSSize(width: 10, height: 10)
        text.isVerticallyResizable = true; text.isHorizontallyResizable = false
        text.autoresizingMask = [.width]; text.textContainer?.widthTracksTextView = true
        text.minSize = NSSize(width: 0, height: 106); text.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        text.isAutomaticQuoteSubstitutionEnabled = false; text.isAutomaticDashSubstitutionEnabled = false
        scroll.documentView = text
        return scroll
    }
    func updateNSView(_ scroll: NSScrollView, context: Context) {
        guard let text = scroll.documentView as? EndTextView else { return }
        registry.register(id, text); text.setAccessibilityElement(true); text.setAccessibilityRole(.textArea); text.setAccessibilityLabel("Notes")
        if text.string != value { text.string = value }
        text.textColor = color; text.onChange = change
    }
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSScrollView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 560, height: proposal.height ?? 108)
    }
}

import AppKit
import SwiftUI

@MainActor final class TodoWindow: NSWindow {
    var onBack: (() -> Void)?
    var onEscape: (() -> Void)?
    override func performKeyEquivalent(with event: NSEvent) -> Bool {
        if event.modifierFlags.contains(.command), event.charactersIgnoringModifiers == "[" { onBack?(); return true }
        if event.keyCode == 53 { onEscape?(); return true }
        return super.performKeyEquivalent(with: event)
    }
    override func keyDown(with event: NSEvent) {
        if event.keyCode == 53 { onEscape?() } else { super.keyDown(with: event) }
    }
}

@MainActor final class WindowHost {
    let registry = PartRegistry()
    var window: TodoWindow?
    var hosting: NSHostingView<TodoRoot>?
    var store: TodoStore?
    var toolbarController: TodoToolbar?
    let seeds: URL
    init(seeds: URL) { self.seeds = seeds }
    func mount(component: String, props: [String: Any], width: CGFloat, height: CGFloat, scheme: String) throws {
        window?.orderOut(nil); window?.close(); hosting = nil; registry.views.removeAll()
        let store = try TodoStore(component: component, props: props, scheme: scheme, seeds: seeds)
        self.store = store
        let w = TodoWindow(contentRect: NSRect(x: 80, y: 80, width: width, height: height),
                           styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
        w.isReleasedWhenClosed = false; w.title = "Todo"; w.animationBehavior = .none
        w.appearance = NSAppearance(named: store.dark ? .darkAqua : .aqua)
        w.onBack = { [weak store] in store?.back() }
        w.onEscape = { [weak store] in store?.adding = false }
        toolbarController = nil
        if component == "todo.app" {
            let controller = TodoToolbar(store: store, registry: registry)
            toolbarController = controller
            let toolbar = NSToolbar(identifier: "todo.toolbar")
            toolbar.delegate = controller; toolbar.allowsUserCustomization = false; toolbar.displayMode = .iconOnly
            w.toolbarStyle = .expanded; w.toolbar = toolbar
            w.titlebarAppearsTransparent = true
            w.backgroundColor = Palette(dark: store.dark).background
        }
        let view = NSHostingView(rootView: TodoRoot(store: store, registry: registry, nativeToolbar: component == "todo.app"))
        view.frame = NSRect(x: 0, y: 0, width: width, height: height)
        w.contentView = view
        w.setContentSize(NSSize(width: width, height: height))
        window = w; hosting = view
        w.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true)
        w.makeFirstResponder(nil)
        if component == "todo.app" {
            settle(0.03)
            guard let toolbarView = nativeToolbarView else { throw HostError("native toolbar did not mount") }
            w.setContentSize(NSSize(width: width, height: height - toolbarView.bounds.height))
        }
    }
    func settle(_ duration: TimeInterval = 0.04) {
        let end = Date(timeIntervalSinceNow: duration)
        repeat { RunLoop.main.run(until: min(end, Date(timeIntervalSinceNow: 0.005))) } while Date() < end
        window?.contentView?.layoutSubtreeIfNeeded(); window?.displayIfNeeded()
    }
    var nativeToolbarView: NSView? {
        var v = registry.find("todo.app.openSettings")?.superview
        while let current = v {
            if NSStringFromClass(type(of: current)).contains("ToolbarView") { return current }
            v = current.superview
        }
        return nil
    }
    var titlebarHeight: CGFloat {
        guard let w = window else { return 0 }
        return w.frame.height - w.contentLayoutRect.height - (nativeToolbarView?.bounds.height ?? 0)
    }
    func frame(of view: NSView) -> NSRect {
        guard let full = window?.contentView?.superview else { return .zero }
        let r = view.convert(view.bounds, to: full)
        return NSRect(x: r.minX, y: full.bounds.height - r.maxY, width: r.width, height: r.height)
    }
    /// Capture the real window view hierarchy into an explicit 1x bitmap, independent of the
    /// display's backingScaleFactor. No screen-recording permission or desktop crop is needed.
    func screenshot(_ path: String) throws {
        guard let full = window?.contentView?.superview else { throw HostError("no mounted window") }
        try capture(full, path: path)
    }
    func capture(_ full: NSView, path: String) throws {
        full.layoutSubtreeIfNeeded(); window?.displayIfNeeded()
        let size = full.bounds.size
        guard let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(size.width), pixelsHigh: Int(size.height), bitsPerSample: 8,
                                         samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0) else { throw HostError("cannot allocate screenshot") }
        rep.size = size
        full.cacheDisplay(in: full.bounds, to: rep)
        guard let data = rep.representation(using: .png, properties: [:]) else { throw HostError("cannot encode PNG") }
        try FileManager.default.createDirectory(at: URL(fileURLWithPath: path).deletingLastPathComponent(), withIntermediateDirectories: true)
        try data.write(to: URL(fileURLWithPath: path))
    }
}
struct HostError: Error, CustomStringConvertible {
    var description: String
    init(_ description: String) { self.description = description }
}

@MainActor final class ProbeSurface: NSView {
    override var isFlipped: Bool { true }
    override func draw(_ dirtyRect: NSRect) {
        NSColor.white.setFill(); bounds.fill()
        NSColor.black.setFill(); NSRect(x: 8, y: 8, width: 32, height: 32).fill()
        NSColor.red.setFill(); NSRect(x: bounds.width - 40, y: bounds.height - 40, width: 32, height: 32).fill()
    }
}

@MainActor final class TodoToolbar: NSObject, NSToolbarDelegate {
    let store: TodoStore
    let registry: PartRegistry
    init(store: TodoStore, registry: PartRegistry) { self.store = store; self.registry = registry }
    private func slot(width: CGFloat) -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: width, height: 48))
        NSLayoutConstraint.activate([
            view.widthAnchor.constraint(equalToConstant: width),
            view.heightAnchor.constraint(equalToConstant: 48)
        ])
        return view
    }
    func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] { [.init("todo.brand"), .flexibleSpace, .init("todo.settings")] }
    func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] { toolbarAllowedItemIdentifiers(toolbar) }
    func toolbar(_ toolbar: NSToolbar, itemForItemIdentifier identifier: NSToolbarItem.Identifier, willBeInsertedIntoToolbar flag: Bool) -> NSToolbarItem? {
        let item = NSToolbarItem(itemIdentifier: identifier)
        if identifier.rawValue == "todo.settings" {
            let button = ActionButton(title: "Settings", target: nil, action: #selector(ActionButton.activate))
            button.target = button; button.invoke = { [weak store] in store?.navigate("settings") }
            button.bezelStyle = .toolbar; button.isBordered = false
            button.image = NSImage(systemSymbolName: "gearshape", accessibilityDescription: nil); button.imagePosition = .imageLeading
            button.setAccessibilityElement(true); button.setAccessibilityRole(.button); button.setAccessibilityLabel("Settings")
            button.frame = NSRect(x: 0, y: 0, width: 107, height: 36)
            registry.register("todo.app.openSettings", button)
            button.contentTintColor = Palette(dark: store.dark).accent
            button.font = .systemFont(ofSize: 14, weight: .medium)
            button.imageHugsTitle = true
            let container = slot(width: 107)
            button.translatesAutoresizingMaskIntoConstraints = false
            container.addSubview(button)
            NSLayoutConstraint.activate([
                button.centerXAnchor.constraint(equalTo: container.centerXAnchor),
                button.centerYAnchor.constraint(equalTo: container.centerYAnchor),
                button.widthAnchor.constraint(equalToConstant: 107),
                button.heightAnchor.constraint(equalToConstant: 36)
            ])
            item.view = container; item.label = "Settings"
        } else {
            let brand = NSTextField(labelWithString: "Todo")
            brand.font = .systemFont(ofSize: 18, weight: .semibold)
            brand.textColor = Palette(dark: store.dark).accent
            registry.register("todo.app.brand", brand)
            let icon = NSImageView(image: NSImage(systemSymbolName: "checkmark.square", accessibilityDescription: nil)!)
            icon.contentTintColor = Palette(dark: store.dark).accent
            icon.setAccessibilityElement(false)
            registry.register("todo.app.brandIcon", icon)
            let stack = NSStackView(views: [icon, brand])
            stack.orientation = .horizontal; stack.alignment = .centerY; stack.spacing = 6
            let container = slot(width: 124)
            stack.translatesAutoresizingMaskIntoConstraints = false
            container.addSubview(stack)
            NSLayoutConstraint.activate([
                stack.centerXAnchor.constraint(equalTo: container.centerXAnchor),
                stack.centerYAnchor.constraint(equalTo: container.centerYAnchor),
                icon.widthAnchor.constraint(equalToConstant: 20),
                icon.heightAnchor.constraint(equalToConstant: 20)
            ])
            item.view = container; item.label = "Todo"
        }
        return item
    }
}

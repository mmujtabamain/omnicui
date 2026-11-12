import AppKit
import SwiftUI

struct RPCError: Error { let code: Int; let message: String }

@MainActor final class Adapter {
    let host: WindowHost
    init() {
        // target.yaml runs in this directory; the app owns the canonical fixture copy.
        host = WindowHost(seeds: URL(fileURLWithPath: "../../../apps/macos/Resources/seeds.json").standardizedFileURL)
    }
    func string(_ p: [String: Any], _ key: String) throws -> String {
        guard let value = p[key] as? String else { throw RPCError(code: -32602, message: "missing or invalid \(key)") }
        return value
    }
    func viewport(_ p: [String: Any]) throws -> [String: CGFloat] {
        guard let v = p["viewport"] as? [String: CGFloat], let width = v["width"], let height = v["height"],
              width > 0, height > 0, width <= 8192, height <= 8192 else { throw RPCError(code: -32602, message: "invalid viewport") }
        return v
    }
    func locate(_ part: String) throws -> NSView {
        guard let view = host.registry.find(part) else { throw RPCError(code: -32002, message: "no part \(part)") }
        return view
    }
    func bounds(_ view: NSView) -> [String: CGFloat] {
        let r = host.frame(of: view)
        return ["x": r.minX, "y": r.minY, "width": r.width, "height": r.height]
    }
    func focused(_ view: NSView) -> Bool {
        guard let responder = view.window?.firstResponder else { return false }
        if responder === view { return true }
        if let field = view as? NSTextField { return field.currentEditor() === responder }
        return false
    }
    func read(_ p: [String: Any]) throws -> Any {
        let view = try locate(try string(p, "part"))
        switch try string(p, "field") {
        case "text":
            if let f = view as? NSTextField { return f.stringValue }
            if let t = view as? NSTextView { return t.string }
            if let b = view as? NSButton { return b.title }
            return view.accessibilityLabel() ?? ""
        case "name": return view.accessibilityLabel() ?? NSNull() as Any
        case "focused": return focused(view)
        case "disabled": return !((view as? NSControl)?.isEnabled ?? true)
        case "selected":
            if let segment = view as? SegmentPart { return segment.control?.isSelected(forSegment: segment.index) ?? false }
            return false
        case "checked":
            if let s = view as? ActionSwitch { return s.accessibilityValue() ?? NSNull() }
            if let b = view as? NSButton, b.accessibilityRole() == .checkBox { return b.state == .on }
            return NSNull()
        case "value": return view.accessibilityValue() ?? NSNull()
        case "role":
            let role = view.accessibilityRole()?.rawValue ?? "AXUnknown"
            if view.accessibilityRoleDescription() == "status" { return "status" }
            return ["AXButton": "button", "AXCheckBox": "checkbox", "AXSwitch": "switch", "AXTextField": "textbox",
                    "AXTextArea": "textbox", "AXHeading": "heading", "AXStaticText": "text", "AXGroup": "generic"][role] ?? role
        default: throw RPCError(code: -32003, message: "unknown read field")
        }
    }
    func sendKey(_ key: String) throws {
        guard let window = host.window else { throw HostError("no window") }
        let pieces = key.split(separator: "+").map(String.init)
        let last = pieces.last ?? key
        var flags: NSEvent.ModifierFlags = []
        if pieces.contains("Meta") { flags.insert(.command) }
        if pieces.contains("Control") { flags.insert(.control) }
        if pieces.contains("Alt") { flags.insert(.option) }
        if pieces.contains("Shift") { flags.insert(.shift) }
        let chars = ["Enter": "\r", "Space": " ", "Escape": "\u{1b}", "BracketLeft": "[", "Tab": "\t", "Backspace": "\u{7f}"][last] ?? last
        let code: UInt16 = ["Enter": 36, "Space": 49, "Escape": 53, "BracketLeft": 33, "Tab": 48, "Backspace": 51][last] ?? 0
        for type: NSEvent.EventType in [.keyDown, .keyUp] {
            guard let event = NSEvent.keyEvent(with: type, location: .zero, modifierFlags: flags,
                                              timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber,
                                              context: nil, characters: chars, charactersIgnoringModifiers: chars, isARepeat: false, keyCode: code) else { throw HostError("cannot make key event") }
            NSApp.sendEvent(event)
        }
    }
    func mouse(_ type: NSEvent.EventType, at point: NSPoint) throws {
        guard let w = host.window, let event = NSEvent.mouseEvent(with: type, location: point, modifierFlags: [], timestamp: ProcessInfo.processInfo.systemUptime,
                                                                windowNumber: w.windowNumber, context: nil, eventNumber: 0, clickCount: 1, pressure: 1) else { throw HostError("cannot make mouse event") }
        w.sendEvent(event)
    }
    func step(_ method: String, _ p: [String: Any], settle: Bool = true) throws {
        switch method {
        case "press":
            let v = try locate(try string(p, "part"))
            if let control = v as? NSControl { control.performClick(nil) }
            else if !v.accessibilityPerformPress() { throw HostError("part does not support accessibility press") }
        case "focus":
            let v = try locate(try string(p, "part"))
            guard v.window?.makeFirstResponder(v) == true else { throw HostError("part cannot take keyboard focus") }
        case "key":
            if let part = p["part"] as? String { _ = try locate(part) }
            try sendKey(try string(p, "key"))
        case "type":
            let v = try locate(try string(p, "part"))
            guard v.window?.makeFirstResponder(v) == true else { throw HostError("field cannot take keyboard focus") }
            for c in try string(p, "text") { try sendKey(String(c)); host.settle(0.001) }
        case "hover", "drag":
            let v = try locate(try string(p, "part"))
            let r = v.convert(v.bounds, to: nil); let point = NSPoint(x: r.midX, y: r.midY)
            if method == "hover" { try mouse(.mouseMoved, at: point) }
            else {
                try mouse(.leftMouseDown, at: point)
                let end = NSPoint(x: point.x + (p["dx"] as? CGFloat ?? 0), y: point.y - (p["dy"] as? CGFloat ?? 0))
                try mouse(.leftMouseDragged, at: end); try mouse(.leftMouseUp, at: end)
            }
        default: throw RPCError(code: -32601, message: "unknown step \(method)")
        }
        if settle { host.settle(host.store?.adding == true ? 0.25 : 0.04) }
    }
    func handle(_ method: String, _ p: [String: Any]) throws -> Any {
        switch method {
        case "advertise":
            return ["protocol": "1", "platform": ["os": "macos", "version": ProcessInfo.processInfo.operatingSystemVersionString],
                    "can": ["mount", "press", "focus", "key", "type", "hover", "drag", "read", "events", "bounds", "role", "focused", "selected", "screenshot", "probe", "trace", "trace.scale", "trace.opacity", "record"],
                    "cannot": ["do.swipe": "not a touch target"]]
        case "probe":
            let viewport = try viewport(p)
            try host.mount(component: "todo.list", props: [:], width: viewport["width"]!, height: viewport["height"]!, scheme: "light")
            let view = ProbeSurface(frame: host.hosting!.frame)
            host.window!.contentView = view
            host.settle(); try host.capture(view, path: try string(p, "path"))
            return p["path"]!
        case "mount":
            let component = try string(p, "component")
            guard ["todo.app", "todo.list", "todo.detail", "todo.settings"].contains(component) else { throw RPCError(code: -32001, message: "unknown component \(component)") }
            let viewport = try viewport(p)
            try host.mount(component: component, props: p["props"] as? [String: Any] ?? [:], width: viewport["width"]!, height: viewport["height"]!, scheme: p["colorScheme"] as? String ?? "light")
            host.settle(0.08); return NSNull()
        case "press", "focus", "key", "type", "hover", "drag": try step(method, p); return NSNull()
        case "read": return try read(p)
        case "bounds": return bounds(try locate(try string(p, "part")))
        case "events": let events = host.store?.events ?? []; host.store?.events.removeAll(); return events
        case "screenshot":
            host.settle(); try host.screenshot(try string(p, "path"))
            return ["path": p["path"]!, "regions": [["kind": "titlebar", "x": 0, "y": 0, "width": host.window!.frame.width, "height": host.titlebarHeight]]]
        case "trace": return try trace(p)
        case "record": return try record(p)
        default: throw RPCError(code: -32601, message: "unknown method \(method)")
        }
    }
    func trace(_ p: [String: Any]) throws -> [[String: Double]] {
        var samples: [[String: Double]] = []
        let part = try string(p, "part")
        var started: TimeInterval?
        func sample() {
            guard let view = host.registry.find(part) else { return }
            let now = ProcessInfo.processInfo.systemUptime
            if started == nil { started = now }
            let r = host.frame(of: view)
            var s: [String: Double] = ["ms": (now - started!) * 1000, "x": r.minX, "y": r.minY]
            if p["point"] as? String == "center" { s["x"] = r.midX; s["y"] = r.midY }
            if p["property"] as? String == "scale" { s["value"] = r.width / max(1, view.bounds.width) }
            if p["property"] as? String == "opacity" {
                var opacity: CGFloat = 1; var node: NSView? = view
                while let n = node { opacity *= n.alphaValue; node = n.superview }
                s["value"] = opacity
            }
            samples.append(s)
        }
        let timer = Timer.scheduledTimer(withTimeInterval: 1 / 60, repeats: true) { _ in MainActor.assumeIsolated { sample() } }
        defer { timer.invalidate() }
        for s in p["steps"] as? [[String: Any]] ?? [] { try step(try string(s, "method"), s, settle: false); host.settle(0.3) }
        host.settle(0.15)
        guard !samples.isEmpty else { throw RPCError(code: -32002, message: "no part \(part) on any frame") }
        return samples
    }
    func record(_ p: [String: Any]) throws -> [String: Any] {
        let path = try string(p, "path")
        var frames: [[String: Any]] = []; var inputs: [Double] = []
        let start = ProcessInfo.processInfo.systemUptime
        func shot() throws {
            let ms = (ProcessInfo.processInfo.systemUptime - start) * 1000
            let file = URL(fileURLWithPath: path).appendingPathComponent("\(frames.count).png").path
            try host.screenshot(file); frames.append(["ms": ms, "file": file])
        }
        for _ in 0..<4 { try shot(); host.settle(0.016) }
        for s in p["steps"] as? [[String: Any]] ?? [] {
            inputs.append((ProcessInfo.processInfo.systemUptime - start) * 1000)
            try step(try string(s, "method"), s, settle: false)
            let end = Date(timeIntervalSinceNow: 0.35)
            repeat { host.settle(0.016); try shot() } while Date() < end
        }
        for _ in 0..<4 { try shot(); host.settle(0.016) }
        return ["frames": frames, "inputs": inputs, "held": false]
    }
}

@main struct AdapterMain {
    @MainActor static func main() {
        NSApplication.shared.setActivationPolicy(.accessory)
        NSApplication.shared.finishLaunching()
        let adapter = Adapter()
        DispatchQueue.global(qos: .userInitiated).async {
          while let line = readLine() {
            DispatchQueue.main.sync { autoreleasepool {
                var id: Any = NSNull()
                var reply: [String: Any] = ["jsonrpc": "2.0"]
                do {
                    guard let data = line.data(using: .utf8), let request = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
                        throw RPCError(code: -32700, message: "parse error")
                    }
                    id = request["id"] ?? NSNull()
                    guard let method = request["method"] as? String else { throw RPCError(code: -32601, message: "missing method") }
                    reply["result"] = try adapter.handle(method, request["params"] as? [String: Any] ?? [:])
                } catch {
                    let e = error as? RPCError
                    reply["error"] = ["code": e?.code ?? -32003, "message": e?.message ?? String(describing: error)]
                }
                reply["id"] = id
                if let data = try? JSONSerialization.data(withJSONObject: reply, options: [.sortedKeys]) {
                    FileHandle.standardOutput.write(data); FileHandle.standardOutput.write(Data([10]))
                }
            } }
          }
          DispatchQueue.main.async { NSApp.terminate(nil) }
        }
        NSApp.run()
    }
}

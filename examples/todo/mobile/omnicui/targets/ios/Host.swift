import UIKit
import SwiftUI
import TodoUI
import Darwin

struct RPCFailure: Error {
    let code: Int
    let message: String
    init(_ message: String) { self.code = -32003; self.message = message }
    init(_ code: Int, _ message: String) { self.code = code; self.message = message }
}

@MainActor final class Host: NSObject, UIApplicationDelegate {
    var window: UIWindow!
    var hosting: UIHostingController<AnyView>?
    var store: TodoStore?
    var component = ""
    var emitted: [[String: Any]] = []
    var viewport = CGSize(width: 360, height: 640)
    var keyboardFrame: CGRect = .zero

    func application(_ application: UIApplication, didFinishLaunchingWithOptions options: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        // Test-host only: enable the same AX runtime used by automation tools.
        // SwiftUI lazily loads its accessibility nodes when automation is enabled.
        if let library = dlopen("/usr/lib/libAccessibility.dylib", RTLD_NOW),
           let symbol = dlsym(library, "_AXSSetAutomationEnabled") {
            unsafeBitCast(symbol, to: (@convention(c) (Int32) -> Void).self)(1)
        }
        window = UIWindow(frame: UIScreen.main.bounds)
        window.rootViewController = UIViewController()
        window.makeKeyAndVisible()
        NotificationCenter.default.addObserver(forName: UIResponder.keyboardDidChangeFrameNotification, object: nil, queue: .main) { [weak self] note in
            MainActor.assumeIsolated { self?.keyboardFrame = note.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? CGRect ?? .zero }
        }
        NotificationCenter.default.addObserver(forName: UIResponder.keyboardDidHideNotification, object: nil, queue: .main) { [weak self] _ in
            MainActor.assumeIsolated { self?.keyboardFrame = .zero }
        }
        let port = UInt16(ProcessInfo.processInfo.environment["OMNICUI_PORT"]!)!
        DispatchQueue.global(qos: .userInitiated).async { [self] in serve(port) }
        return true
    }

    nonisolated func serve(_ port: UInt16) {
        let fd = socket(AF_INET, SOCK_STREAM, 0)
        var addr = sockaddr_in()
        addr.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = port.bigEndian
        addr.sin_addr.s_addr = inet_addr("127.0.0.1")
        let connected = withUnsafePointer(to: &addr) { p in
            p.withMemoryRebound(to: sockaddr.self, capacity: 1) { connect(fd, $0, socklen_t(MemoryLayout<sockaddr_in>.size)) }
        }
        guard connected == 0 else { exit(3) }
        let stream = FileHandle(fileDescriptor: fd, closeOnDealloc: true)
        var buffer = Data()
        while true {
            let chunk = stream.availableData
            if chunk.isEmpty { exit(0) }
            buffer.append(chunk)
            while let end = buffer.firstIndex(of: 10) {
                let line = buffer.prefix(upTo: end); buffer.removeSubrange(...end)
                var response: [String: Any] = ["jsonrpc": "2.0", "id": NSNull()]
                do {
                    let parsed: Any
                    do { parsed = try JSONSerialization.jsonObject(with: line) }
                    catch { throw RPCFailure(-32700, "invalid JSON") }
                    guard let request = parsed as? [String: Any], request["jsonrpc"] as? String == "2.0",
                          let method = request["method"] as? String else { throw RPCFailure(-32600, "invalid JSON-RPC request") }
                    response["id"] = request["id"] ?? NSNull()
                    let params = request["params"] as? [String: Any] ?? [:]
                    var result: Result<Any, Error>!
                    DispatchQueue.main.sync { result = Result { try self.handle(method, params) } }
                    // Let SwiftUI transactions, navigation and sheets finish before replying.
                    // Reads are synchronous and do not pay this wait.
                    if ["mount", "press", "type", "focus", "do", "probe"].contains(method) {
                        let settled = DispatchSemaphore(value: 0)
                        DispatchQueue.main.asyncAfter(deadline: .now() + (method == "mount" ? 0.20 : 0.65)) {
                            self.window.layoutIfNeeded()
                            _ = self.objects()
                            settled.signal()
                        }
                        settled.wait()
                    }
                    response["result"] = try result.get()
                } catch let error as RPCFailure {
                    response["error"] = ["code": error.code, "message": error.message]
                } catch { response["error"] = ["code": -32003, "message": String(describing: error)] }
                do {
                    var data = try JSONSerialization.data(withJSONObject: response, options: [.sortedKeys, .fragmentsAllowed])
                    data.append(10); try stream.write(contentsOf: data)
                } catch { exit(4) }
            }
        }
    }

    func mount(_ p: [String: Any]) throws {
        component = p["component"] as? String ?? ""
        guard ["todo.app", "todo.list", "todo.detail", "todo.settings"].contains(component) else { throw RPCFailure(-32001, "no component \(component)") }
        let props = p["props"] as? [String: Any] ?? [:]
        let size = p["viewport"] as? [String: Any] ?? [:]
        viewport = CGSize(width: size["width"] as? Double ?? 360, height: size["height"] as? Double ?? 640)
        window.frame = CGRect(origin: .zero, size: viewport)
        window.endEditing(true)
        if let old = hosting {
            old.dismiss(animated: false)
            old.view.removeFromSuperview()
        }
        emitted = []
        let s = TodoStore(seed: props["seed"] as? String ?? "three", theme: props["theme"] as? String ?? "system", showCompleted: props["showCompleted"] as? Bool ?? true)
        s.onEvent = { [weak self] name, fields in self?.emitted.append(["name": name, "fields": fields]) }
        store = s
        window.overrideUserInterfaceStyle = p["colorScheme"] as? String == "dark" ? .dark : .light
        let view = TodoRoot(store: s, component: component, taskID: props["taskId"] as? String ?? "t1")
        let host = UIHostingController(rootView: AnyView(view))
        window.rootViewController = host
        host.view.frame = CGRect(origin: .zero, size: viewport)
        // This surface is app content only; system status/home chrome is outside it.
        host.safeAreaRegions = []
        hosting = host
        host.view.layoutIfNeeded()
        _ = objects()
    }

    func objects() -> [NSObject] {
        var found: [NSObject] = []; var seen = Set<ObjectIdentifier>()
        func visit(_ obj: NSObject, _ depth: Int) {
            guard depth < 50, seen.insert(ObjectIdentifier(obj)).inserted else { return }
            found.append(obj)
            if let view = obj as? UIView {
                guard !view.isHidden, view.alpha > 0 else { return }
                for child in view.subviews { visit(child, depth + 1) }
            }
            if let elements = obj.accessibilityElements {
                for case let child as NSObject in elements { visit(child, depth + 1) }
            } else {
                let n = obj.accessibilityElementCount()
                if n > 0 && n < 2000 {
                    for i in 0..<n { if let child = obj.accessibilityElement(at: i) as? NSObject { visit(child, depth + 1) } }
                }
            }
        }
        visit(window, 0)
        return found
    }
    func identifier(_ obj: NSObject) -> String? {
        guard obj.responds(to: NSSelectorFromString("accessibilityIdentifier")) else { return nil }
        return obj.value(forKey: "accessibilityIdentifier") as? String
    }
    func part(_ id: String) throws -> NSObject {
        let all = objects().sorted { ($0.isAccessibilityElement ? 0 : 1) < ($1.isAccessibilityElement ? 0 : 1) }
        let suffix = id.split(separator: ".").last.map(String.init) ?? id
        if let result = all.first(where: { obj in
            let tag = identifier(obj) ?? ""
            return tag == id || (id.hasPrefix("todo.app.") && tag.hasPrefix("todo.") && tag.hasSuffix("." + suffix))
        }) { return result }
        let tags = all.compactMap { o -> String? in
            guard let tag = identifier(o), !tag.isEmpty else { return nil }
            return "\(tag) [\(type(of:o))]"
        }
        throw RPCFailure(-32002, "no part \(id); mounted: \(tags.joined(separator: ", "))")
    }
    func frame(_ obj: NSObject) -> CGRect {
        if let v = obj as? UIView { return v.convert(v.bounds, to: hosting?.view) }
        return hosting!.view.convert(obj.accessibilityFrame, from: nil)
    }
    func box(_ r: CGRect) -> [String: Double] { ["x": r.origin.x, "y": r.origin.y, "width": r.width, "height": r.height] }
    func activate(_ obj: NSObject) throws {
        guard !obj.accessibilityTraits.contains(.notEnabled) else { return }
        if let input = textInput(obj) {
            guard input.becomeFirstResponder() else { throw RPCFailure("text input refused focus") }
            input.selectedTextRange = input.textRange(from: input.endOfDocument, to: input.endOfDocument)
            return
        }
        // UISegmentedControl exposes virtual segment accessibility elements,
        // not UIControls. Dispatch the native control's valueChanged action.
        for segmented in objects().compactMap({ $0 as? UISegmentedControl }) {
            if let index = (0..<segmented.numberOfSegments).first(where: {
                guard let segment = segmented.accessibilityElement(at: $0) as? NSObject else { return false }
                return segment === obj || (identifier(segment) != nil && identifier(segment) == identifier(obj))
            }) {
                segmented.selectedSegmentIndex = index
                segmented.sendActions(for: .valueChanged)
                return
            }
        }
        if let control = obj as? UIControl, !(control is UISwitch) {
            control.sendActions(for: control.allControlEvents.contains(.primaryActionTriggered) ? .primaryActionTriggered : .touchUpInside)
            return
        }
        if let toggle = obj as? UISwitch { toggle.setOn(!toggle.isOn, animated: false); toggle.sendActions(for: .valueChanged); return }
        guard obj.accessibilityActivate() else { throw RPCFailure("\(type(of:obj)) cannot accessibilityActivate") }
    }
    func textInput(_ obj: NSObject) -> (UIView & UITextInput)? {
        if let v = obj as? (UIView & UITextInput) { return v }
        let r = frame(obj)
        return objects().compactMap { $0 as? (UIView & UITextInput) }.first { frame($0).intersects(r) }
    }
    @discardableResult func render(_ path: String) throws -> [[String: Any]] {
        guard let view = hosting?.view else { throw RPCFailure("nothing mounted") }
        view.layoutIfNeeded()
        let format = UIGraphicsImageRendererFormat(); format.scale = 1; format.opaque = true
        let renderer = UIGraphicsImageRenderer(size: viewport, format: format)
        var regions: [[String: Any]] = []
        let keyboard = view.convert(keyboardFrame, from: nil).intersection(CGRect(origin: .zero, size: viewport))
        let image = renderer.image { ctx in
            UIColor.systemBackground.setFill(); ctx.fill(CGRect(origin: .zero, size: viewport))
            window.drawHierarchy(in: window.bounds, afterScreenUpdates: true)
            // UIKit may place input UI in another window. Composite those windows
            // at their native point frames, clipped to this viewport, without scaling.
            let windows = window.windowScene?.windows ?? UIApplication.shared.windows
            for overlay in windows.sorted(by: { $0.windowLevel < $1.windowLevel }) where overlay !== window && !overlay.isHidden && overlay.alpha > 0 {
                let rect = overlay.convert(overlay.bounds, to: window)
                guard rect.intersects(CGRect(origin: .zero, size: viewport)) else { continue }
                let drawn = overlay.drawHierarchy(in: rect, afterScreenUpdates: true)
                if drawn && !keyboard.isNull && keyboard.height > 0 && rect.intersects(keyboard) && regions.isEmpty {
                    var region: [String: Any] = box(keyboard)
                    region["kind"] = "keyboard"; regions.append(region)
                }
            }
        }
        try image.pngData()!.write(to: URL(fileURLWithPath: path))
        return regions
    }
    func handle(_ method: String, _ p: [String: Any]) throws -> Any {
        switch method {
        case "advertise":
            return ["protocol": "1", "platform": ["os": "ios", "version": UIDevice.current.systemVersion],
                    "can": ["mount", "press", "type", "read", "events", "role", "selected", "focused", "bounds", "screenshot", "probe", "do.back"],
                    "cannot": ["focus": "In-process UIKit cannot move hardware keyboard focus to arbitrary SwiftUI controls through a public API.", "key": "No public in-process API injects hardware key events; UIKeyInput is used only for text insertion.", "hover": "Touch interface; no pointer injection.", "drag": "No public in-process touch injection.", "trace": "Per-frame trace transport is not implemented.", "record": "Frame recording transport is not implemented."] ] as [String: Any]
        case "mount": try mount(p); return NSNull()
        case "events": let result = emitted; emitted = []; return result
        case "probe":
            let size = p["viewport"] as! [String: Double]
            viewport = CGSize(width: size["width"]!, height: size["height"]!)
            let probe = UIView(frame: CGRect(origin: .zero, size: viewport)); probe.backgroundColor = .white
            for (r, color) in [(CGRect(x: 8, y: 8, width: 32, height: 32), UIColor.black), (CGRect(x: viewport.width - 40, y: viewport.height - 40, width: 32, height: 32), UIColor.red)] {
                let v = UIView(frame: r); v.backgroundColor = color; probe.addSubview(v)
            }
            let host = UIHostingController(rootView: AnyView(Color.white))
            host.view = probe; hosting = host
            window.frame = CGRect(origin: .zero, size: viewport)
            window.rootViewController = host
            let path = p["path"] as! String; try render(path); return path
        case "screenshot":
            let path = p["path"] as! String
            // SpringBoard status/home chrome is outside the render surface.
            return ["path": path, "regions": try render(path)]
        case "do":
            guard p["name"] as? String == "back" else { throw RPCFailure(-32601, "unsupported do") }
            if let presented = hosting?.presentedViewController { presented.dismiss(animated: true); return NSNull() }
            let all = objects()
            guard let back = all.first(where: { (identifier($0) ?? "").hasSuffix(".detailBack") || (identifier($0) ?? "").hasSuffix(".settingsBack") }) else { throw RPCFailure("no navigation back control") }
            try activate(back); return NSNull()
        case "bounds":
            guard let obj = try? part(p["part"] as! String) else { return box(.zero) }
            return box(frame(obj))
        case "press": try activate(try part(p["part"] as! String)); return NSNull()
        case "type":
            let obj = try part(p["part"] as! String)
            guard let input = textInput(obj) else { throw RPCFailure("part is not a native text input") }
            input.becomeFirstResponder()
            input.selectedTextRange = input.textRange(from: input.endOfDocument, to: input.endOfDocument)
            for character in p["text"] as! String { input.insertText(String(character)) }
            return NSNull()
        case "read":
            let obj = try part(p["part"] as! String)
            let field = p["field"] as! String
            switch field {
            case "role":
                if obj is UISwitch || objects().compactMap({ $0 as? UISwitch }).contains(where: { frame($0).intersects(frame(obj)) }) { return "switch" }
                if textInput(obj) != nil { return "textbox" }
                let traits = obj.accessibilityTraits
                if traits.contains(.header) { return "heading" }
                if traits.contains(.toggleButton) { return "checkbox" }
                if traits.contains(.button) { return "button" }
                if traits.contains(.updatesFrequently) { return "status" }
                return "generic"
            case "name": return obj.accessibilityLabel ?? ""
            case "text":
                if let input = textInput(obj) { return input.text(in: input.textRange(from: input.beginningOfDocument, to: input.endOfDocument)!) ?? "" }
                return obj.accessibilityLabel ?? ""
            case "selected": return obj.accessibilityTraits.contains(.selected)
            case "checked":
                if let toggle = obj as? UISwitch { return toggle.isOn }
                return ["1", "On", "on", "true"].contains(obj.accessibilityValue ?? "")
            case "disabled":
                if let control = obj as? UIControl { return !control.isEnabled }
                return obj.accessibilityTraits.contains(.notEnabled)
            case "focused": return textInput(obj)?.isFirstResponder ?? (obj as? UIView)?.isFirstResponder ?? false
            default: throw RPCFailure("unsupported read field \(field)")
            }
        default: throw RPCFailure(-32601, "unsupported method \(method)")
        }
    }
}

UIApplicationMain(CommandLine.argc, CommandLine.unsafeArgv, nil, NSStringFromClass(Host.self))

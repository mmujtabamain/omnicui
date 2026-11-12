import AppKit

@main struct TodoApplication {
    @MainActor static func main() throws {
        let app = NSApplication.shared
        app.setActivationPolicy(.regular)
        let menu = NSMenu()
        let appItem = NSMenuItem(); menu.addItem(appItem)
        let appMenu = NSMenu(); appItem.submenu = appMenu
        appMenu.addItem(withTitle: "Quit Todo", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        let editItem = NSMenuItem(); menu.addItem(editItem)
        let edit = NSMenu(title: "Edit"); editItem.submenu = edit
        for (title, action, key) in [("Cut", "cut:", "x"), ("Copy", "copy:", "c"), ("Paste", "paste:", "v"), ("Select All", "selectAll:", "a")] {
            edit.addItem(withTitle: title, action: Selector(action), keyEquivalent: key)
        }
        app.mainMenu = menu
        let seeds = Bundle.main.url(forResource: "seeds", withExtension: "json")!
        let host = WindowHost(seeds: seeds)
        let scheme = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? "dark" : "light"
        try host.mount(component: "todo.app", props: [:], width: 1024, height: 720, scheme: scheme)
        let appearance = app.observe(\.effectiveAppearance, options: [.new]) { application, _ in
            let scheme = application.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? "dark" : "light"
            DispatchQueue.main.async { host.store?.scheme = scheme }
        }
        withExtendedLifetime((host, appearance)) { app.run() }
    }
}

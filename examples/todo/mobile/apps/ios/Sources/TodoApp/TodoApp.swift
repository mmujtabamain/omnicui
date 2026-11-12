import SwiftUI
import TodoUI

@main struct TodoApp: App {
    @StateObject private var store = TodoStore()
    var body: some Scene { WindowGroup { TodoRoot(store: store) } }
}

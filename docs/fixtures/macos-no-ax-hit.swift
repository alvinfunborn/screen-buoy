// Build with: swiftc macos-no-ax-hit.swift -o ax-fallback-qa
// A disposable desktop target: AXWindows/focus work, point hit testing has no
// element. It never sends input or makes network requests.
import Cocoa

final class QAApplication: NSApplication {
    override func accessibilityHitTest(_ point: NSPoint) -> Any? { nil }
}

final class CounterView: NSView {
    private var downs = 0
    private var doubles = 0
    override var acceptsFirstResponder: Bool { true }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    override func mouseDown(with event: NSEvent) {
        downs += 1
        if event.clickCount == 2 { doubles += 1 }
        needsDisplay = true
    }
    override func draw(_ dirtyRect: NSRect) {
        NSColor(calibratedRed: 0.12, green: 0.15, blue: 0.22, alpha: 1).setFill()
        bounds.fill()
        let lines = [
            "Screen Buoy: AX fallback QA",
            "Point hit testing deliberately returns no AX element.",
            "Click anywhere here, then use hints + Space + Enter.",
            "Mouse downs: \(downs)",
            "Double clicks: \(doubles)",
        ]
        for (index, line) in lines.enumerated() {
            (line as NSString).draw(at: NSPoint(x: 30, y: bounds.height - 65 - CGFloat(index * 55)),
                withAttributes: [.font: NSFont.systemFont(ofSize: index > 2 ? 30 : 22),
                                 .foregroundColor: NSColor.white])
        }
    }
}

let app = QAApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 200, y: 200, width: 800, height: 380),
                      styleMask: [.titled, .closable, .miniaturizable, .resizable],
                      backing: .buffered, defer: false)
window.title = "Screen Buoy AX fallback QA"
window.contentView = CounterView()
window.center()
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
app.run()

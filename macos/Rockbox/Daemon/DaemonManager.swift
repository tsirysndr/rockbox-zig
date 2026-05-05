import Foundation

final class DaemonManager {
    static let shared = DaemonManager()

    private let lock = NSLock()
    private var _started = false
    private var _port: Int = 0

    var port: Int { lock.withLock { _port } }

    private init() {}

    func start(musicDir: String? = nil, deviceName: String = "Rockbox") throws -> Int {
        var alreadyStarted = false
        lock.withLock {
            if _started { alreadyStarted = true } else { _started = true }
        }
        if alreadyStarted {
            let p = Int(rb_daemon_port())
            return p > 0 ? p : port
        }

        let result: Int32
        if let dir = musicDir {
            result = dir.withCString { dirPtr in
                deviceName.withCString { namePtr in
                    rb_daemon_start(dirPtr, namePtr)
                }
            }
        } else {
            result = deviceName.withCString { namePtr in
                rb_daemon_start(nil, namePtr)
            }
        }

        guard result > 0 else {
            lock.withLock { _started = false }
            throw DaemonError(code: result)
        }
        lock.withLock { _port = Int(result) }
        return Int(result)
    }

    func stop() {
        rb_daemon_stop()
        lock.withLock { _started = false; _port = 0 }
    }
}

struct DaemonError: LocalizedError {
    let code: Int32
    var errorDescription: String? {
        switch code {
        case -22:  return "Invalid input to Rockbox daemon (null device name)"
        case -110: return "Rockbox daemon timed out during startup (>30 s)"
        case -114: return "Rockbox daemon is already starting or running"
        default:   return "Rockbox daemon failed to start (code \(code))"
        }
    }
}

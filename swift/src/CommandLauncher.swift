//
//  Copyright © Uber Technologies, Inc. All rights reserved.
//

import Foundation

enum Command: String {
    case cleanupStaleFlags = "cleanup-stale-flags"
}

protocol CommandLauncher {
    var command: Command { get }
    func launch(_ args: [String]) throws
}

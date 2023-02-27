enum TestFlag {
    case stale_flag_two
    case stale_flag_three

    var var1: String {
        switch self {
            case  .stale_flag_two, .stale_flag_three:
                return "test"
        }
    }

    var var2: String {
        switch self {
            case  .stale_flag_three, .stale_flag_two:
                return "test"
        }
    }
}

enum TestFlag2 {
    case stale_flag_two
    case stale_flag_three

    var var1: String {
        switch self {
            case  .stale_flag_two, .stale_flag_three:
                return "test"
        }
    }

    var var2: String {
        switch self {
            case  .stale_flag_two, .stale_flag_three:
                return "test"
        }
    }
}

enum TestFlag3 {
    case stale_flag_two
    case stale_flag_three

    var var1: String {
        switch self {
            case  .stale_flag_three, .stale_flag_two:
                return "test"
        }
    }

     var var2: String {
        switch self {
            case  .stale_flag_two, .stale_flag_three:
                return "test"
        }
    }
}


enum TestFlag4 {
    case stale_flag_two
    case stale_flag_three

    var var1: String {
        switch self {
            case .stale_flag_three, .stale_flag_two:
                return "test"
        }
    }

    var var2: String {
        switch self {
            case .stale_flag_three, .stale_flag_two:
                return "test"
        }
    }

    var var3: String {
        switch self {
            case .stale_flag_three, .stale_flag_two:
                return "test"
        }
    }
}

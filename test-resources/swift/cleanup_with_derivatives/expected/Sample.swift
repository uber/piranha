class Sample {


    public static func foobar() -> Bool {
        #if DEBUG
            return true
        #else
            return false
        #endif
    }

}

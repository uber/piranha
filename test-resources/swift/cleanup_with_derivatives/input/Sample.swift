class Sample {


    public static func foobar() -> Bool {
        #if DEBUG
            return true
        #else
            return placeholder_false
        #endif
    }

}

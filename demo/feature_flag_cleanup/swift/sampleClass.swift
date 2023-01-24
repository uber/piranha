class SampleClass {
    init() {
        // Code 1
        if TestEnum.stale_flag.isEnabled && abcd() {
            // Code 2
        } else if abcd() && TestEnum.stale_flag.isEnabled {
            // Code 2
        } else if !TestEnum.stale_flag.isEnabled && abcd() {
            // Code 2
        } else if TestEnum.stale_flag && !true {
            // Code 3
        } else if TestEnum.stale_flag.type && !false {
            // code 4
        } else if TestEnum.stale_flag.isEnabled || abcd() {
            // code 7
        } else if def || TestEnum.stale_flag.isEnabled {
            // code 8
        } else if TestEnum.staled_flag {
            // code 9
        } else if TestEnum.staled_flag.type {
            // code 10
        } 
        // Code 11

        let v1 = false && abcd()
    }
}
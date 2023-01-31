class SampleClass {
    init() {
        // Code 1
        if TestEnum.stale_flag.isEnabled && dummy_func_1() {
            // Code 2
        } else if dummy_func_1() && TestEnum.stale_flag.isEnabled {
            // Code 2
        } else if !TestEnum.stale_flag.isEnabled && dummy_func_1() {
            // Code 2
        } else if TestEnum.stale_flag && !true {
            // Code 3
        } else if TestEnum.stale_flag.type && !false {
            // code 4
        } else if TestEnum.stale_flag.isEnabled || dummy_func_1() {
            // code 7
        } else if dummy_var_1 || TestEnum.stale_flag.isEnabled {
            // code 8
        } else if TestEnum.staled_flag {
            // code 9
        } else if TestEnum.staled_flag.type {
            // code 10
        } 
        // Code 11

        let v1 = false && dummy_func_1()
        let v2 = dummy_func_1() && false 

        if TestEnum.stale_flag.isEnabled {
            dummy_func_1()
        } else if TestEnum.stale_flag.isEnabled{
            dummy_func_2()
        } else if TestEnum.stale_flag.isEnabled{
            dummy_func_3()
        } else {
            dummy_func_4()
        }
    }
}
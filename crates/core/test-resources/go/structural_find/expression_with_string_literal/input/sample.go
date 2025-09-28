package flag

import "fmt"


func a() {
    if exp.BoolValue("staleFlagConst") {
        fmt.Println("true")
    } else {
        fmt.Println("false")
    }
}

func a1() {
    if exp.BoolValue(staleFlagConst) {
        fmt.Println("true")
    } else {
        fmt.Println("false")
    }
}

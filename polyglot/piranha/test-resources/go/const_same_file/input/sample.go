package flag

import "fmt"

const (
	staleFlagConst = "staleFlag"
	normalFlag     = "normalFlag"
)

func a() {
	if exp.BoolValue(staleFlagConst) {
		fmt.Println("true")
	} else {
		fmt.Println("false")
	}
}

func (c *Client) b() {
	enabled, err := exp.BoolValue(staleFlagConst)
	if err != nil {
		fmt.Println(err)
	}

	s, err := exp.StrValue("str")
	if err != nil {
		fmt.Println(err)
	}

	if enabled {
		fmt.Println("enabled")
	} else {
		fmt.Println(staleFlagConst)
	}
}

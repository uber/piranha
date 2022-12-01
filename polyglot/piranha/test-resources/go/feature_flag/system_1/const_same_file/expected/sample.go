package flag

import "fmt"

const (
	normalFlag = "normalFlag"
)

func a() {
	fmt.Println("false")
}

func (c *Client) b() {
	s, err := exp.StrValue("str")
	if err != nil {
		fmt.Println(err)
	}

	fmt.Println("staleFlag")
}

func (c *Client) c(enabled2 bool, enabled3 bool) {
	if enabled2 || enabled3 {
		fmt.Println("enabled")
	}
}

// should not replace the function name
func (c *Client) isEnabled() bool {
	return false
}

func (c *Client) callerMethod() {
	// should not replace isFlagEnabledMethod here
	if c.isFlagEnabledMethod() {
		fmt.Println("enabled")
	} else {
		fmt.Println("disabled")
	}
}

// should not replace the method name
func (c *Client) isFlagEnabledMethod() bool {
	fmt.Println("not enabled")
	return false
}

func callerFunc() {
	// should not replace isFlagEnabledFunc here
	if isFlagEnabledFunc() {
		fmt.Println("enabled")
	} else {
		fmt.Println("disabled")
	}
}

// should not replace the function name
func isFlagEnabledFunc() bool {
	fmt.Println("not enabled")
	return false
}

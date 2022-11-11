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

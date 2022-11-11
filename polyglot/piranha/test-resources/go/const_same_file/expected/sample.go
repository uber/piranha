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

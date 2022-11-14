package flag

import (
	"errors"
	"fmt"
)

const (
	normalFlag = "normalFlag"
)

func (c *Client) unusedVar() (string, error) {
	return "Not enabled", nil
}

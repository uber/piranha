package flag

import (
	"errors"
	"fmt"
)

const (
	staleFlagConst = "staleFlag"
	normalFlag     = "normalFlag"
)

func (c *Client) unusedVar() (string, error) {
	param := "Call Parameter"
	enabled, err := exp.BoolValue(staleFlagConst, param)
	if err != nil {
		return "", errors.New("Error")
	}

	if enabled {
		return "Enabled", nil
	}

	return "Not enabled", nil
}

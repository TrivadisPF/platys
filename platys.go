package main

import "trivadis.com/platys/cmd"

//go:generate go run cmd/init_banner.txt
func main() {
	cmd.Execute()
}

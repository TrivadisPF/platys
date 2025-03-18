package main

import (
	"embed"
	"trivadis.com/platys/cmd"
)

//go:embed assets
var Assets embed.FS

func main() {
	//inject assets into the global variable in cmd package as embed does not work with relative paths
	cmd.Assets = Assets
	cmd.Execute()
}

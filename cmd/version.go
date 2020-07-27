package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(versionCmd)
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print the version number of platys",
	Long:  `Print the version number of platys`,
	Run: func(cmd *cobra.Command, args []string) {
		versionInfo := fmt.Sprintf(
			`Platys - Trivadis Platform in a Box - v %v
https://github.com/trivadispf/platys
Copyright (c) 2018-2020, Trivadis AG`,
			version)

		fmt.Println(versionInfo)
	},
}

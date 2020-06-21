package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(genCmd)
}

var genCmd = &cobra.Command{
	Use:   "gen",
	Short: "Generates all the needed artifacts for the docker-based modern (data) platform",
	Long:  `Generates all the needed artifacts for the docker-based modern (data) platform.    
    		The stack configuration can either be passed as a local file (using the --config-filename option or using the default name 'config.yml') 
			or as an URL
    		referencing a file on the Internet (using the --config-url option).`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Platys v 1.0")
	},
}

package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v2"
	"regexp"
)

var serviceRegex = regexp.MustCompile(`(?P<service>[A-Z0-9_-]+)_enable`)

func init() {
	rootCmd.AddCommand(listServicesCmd)
}

var listServicesCmd = &cobra.Command{
	Use:   "list_services",
	Short: "List the services",
	Long:  `List the services contained in the given version of the platys tool`,
	Run: func(cmd *cobra.Command, args []string) {

		var ymlConfig map[interface{}]interface{}
		err := yaml.Unmarshal([]byte(pullConfig()), &ymlConfig)
		if err != nil {
			panic(err)
		}

		fmt.Println("**********************************************************************************************")
		fmt.Printf("* The following services are available in [ %v : %v ]  * \n", Stack, Version)
		fmt.Println("**********************************************************************************************")

		for k, _ := range ymlConfig {
			services := serviceRegex.FindStringSubmatch(fmt.Sprintf("%v", k))
			if len(services) > 0 {
				fmt.Println(services[1])
			}
		}

	},
}

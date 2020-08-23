package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
	"regexp"
)

// is considered a service any key that matches the regex
var serviceRegex = regexp.MustCompile(`(?P<service>[A-Z0-9_-]+)_enable`)

func init() {
	rootCmd.AddCommand(listServicesCmd)
	listServicesCmd.Flags().StringVarP(&Stack, "stack", "s", "trivadis/platys-modern-data-platform", "stack version to employ")
	listServicesCmd.Flags().StringVarP(&Version, "stack-version", "w", "latest", "version of the stack to employ")

}

var listServicesCmd = &cobra.Command{
	Use:   "list_services",
	Short: "List the services",
	Long:  `List the services contained in the given version of the platys tool`,
	Run: func(cmd *cobra.Command, args []string) {

		var ymlConfig yaml.Node
		err := yaml.Unmarshal([]byte(pullConfig()), &ymlConfig)

		if err != nil {
			panic(err)
		}

		fmt.Println("**********************************************************************************************")
		fmt.Printf("* The following services are available in [ %v : %v ]  * \n", Stack, Version)
		fmt.Println("**********************************************************************************************")

		for _, k := range ymlConfig.Content[0].Content {

			services := serviceRegex.FindStringSubmatch(fmt.Sprintf("%v", k.Value))
			if len(services) > 0 {
				fmt.Println(services[1])
			}
		}

	},
}

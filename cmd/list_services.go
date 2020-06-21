package cmd

import (

	"github.com/spf13/cobra"

)

func init() {
	rootCmd.AddCommand(listServicesCmd)
}

var listServicesCmd = &cobra.Command{
	Use:   "list_services",
	Short: "List the services",
	Long:  `List the services contained in the given version of the platy`,
	Run: func(cmd *cobra.Command, args []string) {
		pullConfig()

	/*	tar_config = pull_config(Stack, Version)

		// extract the config file from the tar in to the current folder
		tar_file = tarfile.open(tar_config)
		tar_file.extractall(path=tempfile.gettempdir())
		tar_file.close()
		yaml = ruamel.yaml.YAML()
		with open(rf'{tempfile.gettempdir()}/config.yml') as file:
		config_yml = yaml.load(file)
		for c in config_yml:
		service = re.search("([A-Z0-9_-]+)_enable", str(c))  // if variable follows regex it's considered a service and will be printed
		if service is not None:
		print(service.group(1))*/
	},
}



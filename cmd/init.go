package cmd

import (
	"fmt"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v2"
	"log"
	"os"
	"strings"
)

var (
	seedConfig     string
	enableServices string
	force          bool
	hwArch         string
)

func init() {
	rootCmd.AddCommand(initCmd)

	initCmd.Flags().StringP("config-file", "c", "config.yml", "The name of the local config file (defaults to config.yml)")
	initCmd.Flags().StringVarP(&enableServices, "--enable-services", "y", "", "List of services to enable in the config file")
	initCmd.Flags().StringVarP(&seedConfig, "--seed-config", "e", "", "the name of a predefined stack to base this new platform on")
	initCmd.Flags().BoolVarP(&force, "--force", "f", false, "If specified, this command will overwrite any existing config file")
	initCmd.Flags().StringVarP(&hwArch, "--hw-arch", "x", "x86-64", "Hardware architecture for the platform")

}

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial config file, if one does not already exists",
	Long: `Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial
		config file, if one does not already exists The stack to use as well as its version need to be passed by the --stack-image-name and --stack-image-version options.
			By default 'config.yml' is used for the name of the config file, which is created by the init`,
	Run: func(cmd *cobra.Command, args []string) {

		var configFile, err = cmd.Flags().GetString("config-file")
		if err != nil {
			panic(err)
		}
		fmt.Println(fmt.Sprintf("Running using config file [%v]", configFile))

		_, err = os.Stat("./" + configFile)

		if err == nil && !force {
			log.Fatal("config.yml already exists if you want to override it use the [-f] option")

		}

		var ymlConfig map[interface{}]interface{}
		err = yaml.Unmarshal([]byte(pullConfig()), &ymlConfig)

		if err != nil {
			panic(err)
		}

		if enableServices != "" {

			var in_array = func(val string, list []string) bool {
				for _, b := range list {
					if b == val {
						return true
					}
				}
				return false
			}

			services := strings.Split(enableServices, ",")

			for s := range services {

				service := services[s] + "_enable"
				_, found := ymlConfig[service]

				if found {
					if Verbose {
						fmt.Println("Enabling service : [%v]", service)
					}
					ymlConfig[service] = true
				}
			}

			for k, _ := range ymlConfig {
				if strings.Contains(k.(string), "platys") || strings.Contains(k.(string), "use_timezone") || strings.Contains(k.(string), "private_docker_repository_name") {
					continue
				}
				if !in_array(strings.Replace(k.(string), "_enable", "", -1), services) {
					delete(ymlConfig, k)
				}

			}

		}

		f, err := os.Create("config.yml")

		if err != nil {
			fmt.Println(err)
			f.Close()
			log.Fatal("Unable to create config file")
		}

		fmt.Println("file written successfully")

		fmt.Fprintln(f, "      platys:")
		for k, v := range ymlConfig[interface{}(string("platys"))].(map[interface{}]interface{}) {
			fmt.Fprintln(f, fmt.Sprintf("        %v: '%v'", k.(string), v.(string)))
		}

		fmt.Fprintln(f, "")

		for k, v := range ymlConfig {
			if k == "platys" {
				continue
			}
			fmt.Fprintln(f, fmt.Sprintf("      %v: %v", k, v))
		}
		err = f.Close()
		if err != nil {
			fmt.Println(err)
			return
		}

	},
}

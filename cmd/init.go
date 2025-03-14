package cmd

import (
	"bytes"
	"fmt"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
	"log"
	"os"
	"regexp"
	"strings"
)

var (
	seedConfig     string
	enableServices string
	force          bool
	hwArch         string
	structure      string
	platformName   string
)

func init() {
	rootCmd.AddCommand(initCmd)

	initCmd.Flags().StringVarP(&enableServices, "enable-services", "y", "", "Comma separated list of services to enable in the config file")
	initCmd.Flags().BoolVarP(&force, "force", "f", false, "If specified, this command will overwrite any existing config file")
	initCmd.Flags().StringVarP(&hwArch, "hw-arch", "x", "x86-64", "Hardware architecture for the platform")
	initCmd.Flags().StringVarP(&seedConfig, "seed-config", "e", "", "The name of a predefined stack to base this new platform on")
	initCmd.Flags().StringP("config-file", "c", "config.yml", "The name of the local config file (defaults to config.yml)")
	initCmd.Flags().StringVarP(&structure, "structure", "b", "", "defines the structure of the generated platform (flat = platform is generate on the level of the config.yml or subfolder = platform is generated into a subfolder)")
	initCmd.Flags().StringVarP(&platformName, "platform-name", "n", "", "the name of the platform to generate.")
	initCmd.Flags().StringVarP(&Stack, "stack", "s", "trivadis/platys-modern-data-platform", "stack version to employ")
	initCmd.Flags().StringVarP(&Version, "stack-version", "w", "latest", "version of the stack to employ")
}

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial config file, if one does not already exists",
	Long: `Initializes the current directory to be the root for a platys platform by creating an initial
config file, if one does not already exists The stack to use as well as its version need to be passed by the --stack and --stack-version options.
By default 'config.yml' is used for the name of the config file, which is created by the init`,
	Run: func(cmd *cobra.Command, args []string) {

		var configFile, err = cmd.Flags().GetString("config-file")
		if err != nil {
			panic(err)
		}
		fmt.Println(fmt.Sprintf("Running using config file [%v]", configFile))

		_, err = os.Stat("./" + configFile)

		if err == nil && !force {
			log.Fatal(fmt.Sprintf("[%s] already exists if you want to override it use the [-f] option", configFile))
		}

		var ymlConfig yaml.Node
		err = yaml.Unmarshal([]byte(pullConfig()), &ymlConfig)

		if err != nil {
			log.Fatal(err)
		}

		currentService := ""
		if enableServices != "" { // services where passed as an argument

			services := strings.Split(enableServices, ",") // separate service by coma
			servicesYml := ymlConfig.Content[0].Content
			updatedYml := make([]*yaml.Node, 0)
			copied := 0

			for i := 0; i < len(servicesYml)-1; i = i + 2 {
				key := servicesYml[i]
				value := servicesYml[i+1]

				if strings.Contains(key.Value, "_enable") { // services are suffixed with enabled
					currentService = strings.Replace(key.Value, "_enable", "", 1)
				}

				if strings.Contains(key.Value, "platys") {
					updatedYml = append(updatedYml, key)
					updatedYml = append(updatedYml, value)
					copied = copied + 2

				} else if strings.Contains(key.Value, "use_timezone") || strings.Contains(key.Value, "private_docker_repository_name") {
					updatedYml = append(updatedYml, key)
					updatedYml = append(updatedYml, value)
					copied = copied + 2

				} else if in_array(currentService, services) && !isServiceProperty(currentService, key) {
					fmt.Println(fmt.Sprintf("Enabling service [%v]", currentService))
					updatedYml = append(updatedYml, key)
					value.Value = "true" // service was requested to be activated change the value to true before adding it
					updatedYml = append(updatedYml, value)
					copied = copied + 2
				} else if isServiceProperty(currentService, key) && in_array(currentService, services) {
					fmt.Println(fmt.Sprintf("grabbing service property for service [%v]", currentService))
					updatedYml = append(updatedYml, key)
					updatedYml = append(updatedYml, value)
					copied = copied + 2
				}

			}

			updatedYml = updatedYml[:copied] //create a new slice by copying the updated one
			ymlConfig.Content[0].Content = updatedYml
		}

		if len(platformName) > 0 {
			updateConfig("platform-name", platformName, &ymlConfig)
		}

		if len(structure) > 0 {

			if structure != "subfolder" && structure != "flat" {
				log.Fatal(fmt.Sprintf("Invalid value for  [structure] received [%v]. Accepted values are [flat|subfolder] "), structure)
			}
			updateConfig("structure", structure, &ymlConfig)
		}

		b, _ := yaml.Marshal(&ymlConfig)
		b = addRootIndent(b, 6)

		file, err := os.OpenFile(configFile, os.O_RDWR|os.O_CREATE, 0755)
		defer file.Close()

		if err != nil {
			log.Fatal(fmt.Sprintf("Unable to open file %v", err))
		}

		err = file.Truncate(0) // clear the contents of the file as to do not have stale data when writing (will append)
		if err != nil {
			log.Fatal(fmt.Sprintf("Unable to clear file [%v] contents [%v]", configFile, err))
		}

		for _, s := range strings.SplitN(string(b), "\n", -1) { // write updated config file
			_, err = file.Write([]byte(s + "\n"))
			if err != nil {
				log.Fatal(err)
			}
		}

		printBanner(configFile)

	},
}

func isServiceProperty(service string, node *yaml.Node) bool {

	//we ignore anything that has 'enable' string since this is related to activating services
	if strings.Contains(node.Value, "_enable") {
		return false
	}
	pattern := fmt.Sprintf(`^%s_[a-z0-9_]+$`, regexp.QuoteMeta(service))
	re := regexp.MustCompile(pattern)
	return re.MatchString(node.Value)
}

func addRootIndent(b []byte, n int) []byte {
	prefix := append([]byte("\n"), bytes.Repeat([]byte(" "), n)...)
	b = append(prefix[1:], b...) // Indent first line
	return bytes.ReplaceAll(b, []byte("\n"), prefix)
}

//TODO: check for possible a cleaner more concise way
// to implement the following methods

func updateConfig(name string, value string, rootNode *yaml.Node) {

	platys := rootNode.Content[0].Content

	for i, n := range platys {

		if n.Value == "platys" {
			for j, sn := range platys[i+1].Content {
				if sn.Value == name {
					platys[i+1].Content[j+1].Value = value
					break
				}

			}
		}
	}

}

package cmd

import (
	"fmt"
	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/mount"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
	"io/ioutil"
	"log"
	"os"
	"strconv"
)

var delEmptyLines bool
var configUrl string
var configFile string

type YAMLFile struct {
	Platys   Platys `yaml:"platys"`
	Services Services
}
type Platys struct {
	PlatformName      string `yaml:"platform-name"`
	StackImageName    string `yaml:"stack-image-name"`
	StackImageVersion string `yaml:"stack-image-version"`
	Structure         string `yaml:"structure"`
}

type Services struct {
	service yaml.Node `yaml:""`
}

func init() {
	rootCmd.AddCommand(genCmd)

	genCmd.Flags().BoolVarP(&delEmptyLines, "del-empty-lines", "l", true, "Remove empty lines from the docker-compose.yml file.")
	genCmd.Flags().StringVarP(&configUrl, "config-url", "u", "", "The URL to a remote config file")
	genCmd.Flags().StringVarP(&configFile, "config-file", "c", "config.yml", "The name of the local config file (defaults to config.yml)")
	genCmd.MarkFlagRequired("base-folder")
}

var genCmd = &cobra.Command{
	Use:   "gen",
	Short: "Generates all the needed artifacts for the docker-based modern (data) platform",
	Long: `Generates all the needed artifacts for the docker-based modern (data) platform.    
    		The stack configuration can either be passed as a local file (using the --config-filename option or using the default name 'config.yml') 
			or as an URL
    		referencing a file on the Internet (using the --config-url option).`,
	Run: func(cmd *cobra.Command, args []string) {

		var services yaml.Node
		var platys YAMLFile

		if configFile == "" {
			log.Fatal("Unable to run command as configFile is null")
			return
		}
		ymlContent, err := ioutil.ReadFile(configFile)
		if err != nil {
			panic(err)
		}
		err = yaml.Unmarshal(ymlContent, &platys)
		err = yaml.Unmarshal(ymlContent, &services)

		if err != nil {
			panic(err)
		}

		for i, n := range services.Content[0].Content {

			if n.Kind == yaml.ScalarNode {
				if max, found := checkNodeLimits(n.Value); found {
					val, err := strconv.Atoi(services.Content[0].Content[i+1].Value)

					if err != nil {
						fmt.Println(fmt.Sprintf("Unable to parse value %v for key %v", val, services.Content[0].Content[i].Value))
					}

					if val > max {
						panic(fmt.Sprintf("Unable to generate config file since because the number of nodes configured for service [%v] -> [%v] is higher than max value [%v])", val, services.Content[0].Content[i].Value, max))
					}

				}
			}

		}

		if platys.Platys.PlatformName == "" || platys.Platys.StackImageName == "" || platys.Platys.StackImageVersion == "" || platys.Platys.Structure == "" {
			log.Fatal("The config file is not properly formatted or missing information please ensure [platform-name], [stack-image-name] and [stack-image-version] are properly configured")
			return
		}

		if Verbose {
			log.Printf("using configuration file [%v] with values:  platform-name: [%v], stack-image-name: [%v]  stack-image-version: [%v], structure [%v]",
				configFile, platys.Platys.PlatformName, platys.Platys.StackImageName, platys.Platys.StackImageVersion, platys.Platys.Structure)
		}

		var currentFolder, _ = os.Getwd()
		var destination = currentFolder // where the gen command will output

		if platys.Platys.Structure == "subfolder" {
			destination = destination + "/" + platys.Platys.PlatformName

			if err := os.MkdirAll(destination, os.ModePerm); err != nil {
				panic(err)
			}
			log.Printf("Generating stack on [%v]", destination)
		}

		var env []string

		if Verbose {
			env = append(env, "VERBOSE=1")
		} else {
			env = append(env, "VERBOSE=0")
		}

		if delEmptyLines {
			env = append(env, "DEL_EMPTY_LINES=1")
		} else {
			env = append(env, "DEL_EMPTY_LINES=0")
		}

		if configUrl != "" {
			env = append(env, "CONFIG_URL="+configUrl)
		}

		cli, ctx := initClient()

		resp, err := cli.ContainerCreate(ctx, &container.Config{
			Image: Stack,
			Tty:   true,
			Env:   env,
		},
			&container.HostConfig{
				Mounts: []mount.Mount{
					{
						Target:   "/tmp/config.yml", // path in the container
						Source:   currentFolder + "/config.yml",
						Type:     mount.TypeBind,
						ReadOnly: false,
					},
					{
						Target:   "/opt/mdps-gen/destination", // path in the container
						Source:   destination,
						Type:     mount.TypeBind,
						ReadOnly: false,
					},
				},
			}, nil, containerName)

		if err != nil {
			panic(err)
		}

		if err := cli.ContainerStart(ctx, resp.ID, types.ContainerStartOptions{}); err != nil {
			panic(err)
		}

		statusCh, errCh := cli.ContainerWait(ctx, resp.ID, container.WaitConditionNotRunning)
		select {
		case err := <-errCh:
			if err != nil {
				panic(err)
			}
		case <-statusCh:
		}

		out, err := cli.ContainerLogs(ctx, resp.ID, types.ContainerLogsOptions{ShowStdout: true})
		if err != nil {
			panic(err)
		}

		log.Print(out)

		stopRemoveContainer(resp.ID, cli, ctx)

	},
}

// Checks that the max amount of nodes for a given service is not higher than the max amount
func checkNodeLimits(nodeName string) (int, bool) {

	nodeLimits := map[string]int{
		"ZOOKEEPER_nodes":             3,
		"KAFKA_broker_nodes":          6,
		"KAFKA_SCHEMA_REGISTRY_nodes": 2,
		"KAFKA_CONNECT_nodes":         3,
		"KAFKA_KSQLDB_nodes":          3,
		"HADOOP_datanodes":            6,
		"DATASTAX_nodes":              3,
		"MOSQUITTO_nodes":             3,
	}

	val, found := nodeLimits[nodeName]

	return val, found

}

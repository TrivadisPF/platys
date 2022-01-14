package cmd

import (
	"bufio"
	"fmt"
	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/mount"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
	"io"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
)

var delEmptyLines bool
var configUrl string
var configFile string
var legacyParams = []string{"stack-image-name", "stack-image-version"}

type YAMLFile struct {
	Platys Platys `yaml:"platys"`
}

type Platys struct {
	PlatformName         string `yaml:"platform-name"`
	PlatformStack        string `yaml:"platform-stack"`
	PlatformStackVersion string `yaml:"platform-stack-version"`
	Structure            string `yaml:"structure"`
}

func init() {
	rootCmd.AddCommand(genCmd)

	genCmd.Flags().BoolVarP(&delEmptyLines, "del-empty-lines", "l", true, "Remove empty lines from the docker-compose.yml file.")
	genCmd.Flags().StringVarP(&configUrl, "config-url", "u", "", "The URL to a remote config file")
	genCmd.Flags().StringVarP(&configFile, "config-file", "c", "config.yml", "The name of the local config file (defaults to config.yml)")
	_ = genCmd.MarkFlagRequired("base-folder")
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
		var err error

		if configUrl != "" { // configUrl was defined attempt to download it and use it
			log.Print(fmt.Sprintf("[configUrl] was defined with value [%v] overwritting config file", configUrl))
			configFile, err = downloadRemoteFile(configUrl)
			if err != nil {
				log.Fatal(err)
			}
		}

		if configFile == "" {
			log.Fatal("Unable to run command as configFile is null")
		}

		ymlContent, err := ioutil.ReadFile(configFile)

		if err != nil {
			log.Fatal(fmt.Sprintf("Unable to continue as the file [%v] cannot be found", configFile))
		}

		err = yaml.Unmarshal(ymlContent, &platys)
		err = yaml.Unmarshal(ymlContent, &services)

		if err != nil {
			log.Fatal(err)
		}

		isPlatysValid(platys, services)

		// set global values for stack and version
		Stack = platys.Platys.PlatformStack
		Version = platys.Platys.PlatformStackVersion

		for i, n := range services.Content[0].Content {

			if n.Kind == yaml.ScalarNode {
				if max, found := isLimited(n.Value); found {
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

		printInfoIfNecessary(platys)

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

		cli, ctx := initClient()

		var containerConfig = container.Config{}
		if runtime.GOOS == "windows" {
			containerConfig = container.Config{
				Image: platys.Platys.PlatformStack + ":" + platys.Platys.PlatformStackVersion,
				Tty:   true,
				Env:   env}

		} else {
			containerConfig = container.Config{
				Image:        platys.Platys.PlatformStack + ":" + platys.Platys.PlatformStackVersion,
				Tty:          true,
				AttachStderr: true,
				AttachStdout: true,
				Env:          env,
				User:         currentUser(),
			}

		}
		fullPathConfigFile, err := filepath.Abs(configFile) // full path in required for mounting files in the container
		if err != nil {
			log.Fatal(err)
		}

		resp, err := cli.ContainerCreate(ctx, &containerConfig,
			&container.HostConfig{

				Mounts: []mount.Mount{
					{
						Target:   "/tmp/config.yml", // path in the container
						Source:   fullPathConfigFile,
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
			log.Fatal(err)
		}

		if err := cli.ContainerStart(ctx, resp.ID, types.ContainerStartOptions{}); err != nil {
			panic(err)
		}

		statusCh, errCh := cli.ContainerWait(ctx, resp.ID, container.WaitConditionNotRunning)
		select {
		case err := <-errCh:
			if err != nil {
				log.Fatal(err)
			}
		case <-statusCh:
		}

		out, err := cli.ContainerLogs(ctx, resp.ID, types.ContainerLogsOptions{ShowStdout: true, ShowStderr: true})
		if err != nil {
			log.Fatal(err)
		}
		defer func(out io.ReadCloser) {
			_ = out.Close()
		}(out)

		scanner := bufio.NewScanner(out)
		for scanner.Scan() {
			fmt.Println(scanner.Text())
		}

		stopRemoveContainer(resp.ID, cli, ctx)

	},
}

// Checks that the max amount of nodes for a given service is not higher than the max amount
func isLimited(nodeName string) (int, bool) {

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

func isPlatysValid(platys YAMLFile, ymlConfig yaml.Node) {

	if platys.Platys.PlatformName == "" || platys.Platys.PlatformStack == "" ||
		platys.Platys.PlatformStackVersion == "" || platys.Platys.Structure == "" {

		if isOlderVersion(ymlConfig) {
			log.Fatal(
				`The config file is not properly formatted: key names [stack-image-name] , [stack-image-version] are legacy parameters and should be manually renamed
					[stack-image-name --> platform-stack] , [stack-image-version -->platform-stack-version ]`)

		} else {
			log.Fatal("The config file is not properly formatted or missing information please ensure [platform-name], [stack-image-name|platform-stack] and [stack-image-version|platform-stack-version] are properly configured")
		}

	}

	if platys.Platys.Structure != "subfolder" && platys.Platys.Structure != "flat" {
		log.Fatal(fmt.Sprintf("Unable to process config file as value for [structure] is invalid, received [%v]. Accepted values are [flat|subfolder] "), platys.Platys.Structure)
	}

}

func printInfoIfNecessary(platys YAMLFile) {
	if Verbose {

		log.Printf("using configuration file [%v] with values:  platform-name: [%v], platform-stack: [%v] platform-stack-version: [%v], structure [%v]",
			configFile, platys.Platys.PlatformName, platys.Platys.PlatformStack, platys.Platys.PlatformStackVersion, platys.Platys.Structure)
	}

}

//Checks wether the YAML file is for an older version of Platys
func isOlderVersion(rootNode yaml.Node) bool {

	platys := rootNode.Content[0].Content

	for i, n := range platys {

		if n.Value == "platys" {
			for _, sn := range platys[i+1].Content {
				if in_array(sn.Value, legacyParams) {
					return true
				}

			}
		}
	}

	return false
}

func downloadRemoteFile(configUrl string) (string, error) {
	return DownloadFile(configUrl)

}

//Downloads a remote config file to a local one
func DownloadFile(url string) (string, error) {

	// Get the data
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Fatal(fmt.Sprintf("Unable to download configuration file. Error -> [%v]", err))
		}
	}(resp.Body)

	// Create the file
	out, err := ioutil.TempFile("", "config*.yml")
	if err != nil {
		return "", err
	}
	defer func(out *os.File) {
		err := out.Close()
		if err != nil {
			log.Fatal(fmt.Sprintf("Unable to write configuration file. Error -> [%v]", err))
		}
	}(out)

	// Write the body to file
	_, err = io.Copy(out, resp.Body)
	return out.Name(), err
}

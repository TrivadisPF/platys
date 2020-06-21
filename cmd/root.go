package cmd

import (
	"archive/tar"
	"bytes"
	"context"
	"fmt"
	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
	"log"

	"github.com/spf13/cobra"
	"io"
	"os"
)

var Stack string
var Version string

var rootCmd = &cobra.Command{
	Use:   "platys",
	Short: "Platys platform generator",
	Long: `Platys modern data platform generator
                Complete documentation is available at https://github.com/TrivadisPF/platys`,
	Run: func(cmd *cobra.Command, args []string) {
		// Do Stuff Here
	},
}

func init() {
	rootCmd.PersistentFlags().StringVarP(&Stack, "stack", "s", "trivadis/platys-modern-data-platform", "stack version to employ")
	rootCmd.PersistentFlags().StringVarP(&Version, "stack-version", "w", "latest", "version of the stack to employ")
}

func er(msg interface{}) {
	fmt.Println("Error:", msg)
	os.Exit(1)
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func pullConfig(){
	// init and start docker container
	/*client = Docker.from_env()
	dp_container = client.containers.run(image = fmt.Sprintf("%s:%s", Stack, ), detach = True, auto_remove = True)

	// copy default config file (with default values to the current folder
	tar_config = tempfile.gettempdir() + '/config.tar'
	f = open(tar_config, 'wb')
	bits, stats = dp_container.get_archive('/opt/mdps-gen/vars/config.yml')

	for chunk in bits:
	f.write(chunk)
	f.close()*/

	//return tar_config


	ctx := context.Background()

	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		panic(err)
	}

	reader, err := cli.ImagePull(ctx, Stack, types.ImagePullOptions{})
	if err != nil {
		panic(err)
	}
	io.Copy(os.Stdout, reader)

	resp, err := cli.ContainerCreate(ctx, &container.Config{
		Image: Stack,

		Tty:   true,

	}, nil, nil, "platys")
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


	reader, _, err = cli.CopyFromContainer(ctx, resp.ID,"/opt/mdps-gen/vars/config.yml")
	if err != nil{
		log.Println(err.Error())
	}
	tr := tar.NewReader(reader)

	for {
		// hdr gives you the header of the tar file
		hdr, err := tr.Next()
		if err == io.EOF {
			// end of tar archive
			break
		}
		if err != nil {
			log.Fatalln(err)
		}
		buf := new(bytes.Buffer)
		buf.ReadFrom(tr)

		// You can use this wholeContent to create new file
		wholeContent := buf.String()

		fmt.Println("Whole of the string of ", hdr.Name ," is ",wholeContent)

	}

	stdcopy.StdCopy(os.Stdout, os.Stderr, out)
	cli.ContainerRemove(ctx, resp.ID,types.ContainerRemoveOptions{
		RemoveVolumes: true,
		RemoveLinks:   true,
		Force:         true,
	})
}



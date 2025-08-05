package cmd

import (
	"fmt"
	"github.com/docker/distribution/context"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
	"github.com/spf13/cobra"
	"io"
	"os"
)

func init() {
	rootCmd.AddCommand(stacksCmd)
	stacksCmd.Flags().StringVarP(&Version, "stack-version", "w", "latest", "version of the stack to employ")

}

var stacksCmd = &cobra.Command{
	Use:   "stacks",
	Short: "Lists the predefined stacks available for the init command",
	Long:  `Lists the predefined stacks available for the init command`,
	Run: func(cmd *cobra.Command, args []string) {

		ctx := context.Background()

		cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
		if err != nil {
			panic(err)
		}

		reader, err := cli.ImagePull(ctx, Stack+":"+Version, image.PullOptions{})
		if err != nil {
			panic(err)
		}
		io.Copy(os.Stdout, reader)

		resp, err := cli.ContainerCreate(ctx, &container.Config{
			Image: Stack + ":" + Version,
			Tty:   true,
			Cmd:   []string{"ls", "/opt/mdps-gen/seed-stacks"},
		}, nil, nil, nil, containerName)

		defer stopRemoveContainer(resp.ID, cli, ctx)

		if err != nil {
			panic(err)
		}

		if err := cli.ContainerStart(ctx, resp.ID, container.StartOptions{}); err != nil {
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

		out, err := cli.ContainerLogs(ctx, resp.ID, container.LogsOptions{ShowStdout: true})
		if err != nil {
			panic(err)
		}

		stdcopy.StdCopy(os.Stdout, os.Stderr, out)
		content, _ := io.ReadAll(out)

		//TODO implement regex filtering of the logs to match seekds
		fmt.Println("Available stacks :")
		fmt.Println(content)

		/*
			stacks = []
			    for line in log:
			        stack = re.search("([A-Z0-9_-]+).yml", str(line))
			        if stack is not None:
			            stacks.append(stack.group(1))

			    print(*stacks)
		*/

	},
}

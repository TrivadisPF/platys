package cmd

import (
	"archive/tar"
	"fmt"
	"github.com/spf13/cobra"
	"io"
	"io/ioutil"
	"log"
	"os"
	"path"
	"path/filepath"
)

var baseFolder string

func init() {
	rootCmd.AddCommand(cleanCmd)
	cleanCmd.Flags().StringVarP(&baseFolder, "base-folder", "f", "", "the path base folder that will be used to clean: container-volume will be appended to the path")
	cleanCmd.MarkFlagRequired("base-folder")
}

var cleanCmd = &cobra.Command{
	Use:   "clean",
	Short: "Cleans the contents in the $PATH/container-volume folder",
	Long:  `Cleans the contents in the $PATH/container-volume folder`,
	Run: func(cmd *cobra.Command, args []string) {

		folder := baseFolder + "/container-volume"
		fmt.Printf("about to delete content of folder : %v \n", folder)

		dir, err := ioutil.ReadDir(folder)

		if err != nil {
			panic(err)
		}
		for _, d := range dir {
			err := os.RemoveAll(path.Join([]string{folder, d.Name()}...))
			if err != nil {
				panic(err)
			}
		}

		reader, _, err := getFile("/opt/mdps-gen/static-data/container-volume")
		fmt.Printf("About to revert to default structure on folder [%v] \n", baseFolder)

		if err != nil {
			panic(err)
		}
		tr := tar.NewReader(reader)

		for {
			header, err := tr.Next()

			switch {

			// if no more files are found return
			case err == io.EOF:
				return

			// return any other error
			case err != nil:
				panic(err)

			// if the header is nil, just skip it (not sure how this happens)
			case header == nil:
				continue
			}

			// the target location where the dir/file should be created
			target := filepath.Join(baseFolder, header.Name)

			// the following switch could also be done using fi.Mode(), not sure if there
			// a benefit of using one vs. the other.
			// fi := header.FileInfo()

			// check the file type
			switch header.Typeflag {

			// if its a dir and it doesn't exist create it
			case tar.TypeDir:
				if _, err := os.Stat(target); err != nil {
					if err := os.MkdirAll(target, 0755); err != nil {
						panic(err)
					}
				}

			// if it's a file create it
			case tar.TypeReg:
				f, err := os.OpenFile(target, os.O_CREATE|os.O_RDWR, os.FileMode(header.Mode))
				if err != nil {
					panic(err)
				}

				// copy over contents
				if _, err := io.Copy(f, tr); err != nil {
					log.Fatal(err)
				}

				// manually close here after each file operation; defering would cause each file close
				// to wait until all operations have completed.
				f.Close()
			}
		}

	},
}

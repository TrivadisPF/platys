# How to use a platys-generated stack without Internet

If you want to make use of platys-generated stack without the internet, then you can use the following steps. 

Generate a stack on a machine with Internet access as documented in [Getting Started with `platys` and `modern-data-platform` stack](./documentation/getting-started.md). You don't have to start the stack, just generate the artefacts. 

Now pull the images from the docker registry by using

```
docker-compose pull
```

One all the images are downloaded, you can export them intot an archive file

```
docker save -o docker-images.tar $(docker-compose config | awk '{if ($1 == "image:") print $2;}')
```

Now copy the archive as well as the generated artefacts to the machine without internet and load the images into the local docker registry

```
docker load -i docker-images.tar
```
   
Set the `DOCKER_HOST_IP` and `PUBLIC_IP` environment variables and start the stack using

```
docker-compose up -d
```
